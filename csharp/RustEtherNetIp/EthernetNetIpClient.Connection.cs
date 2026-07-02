using System;
using System.Linq;
using System.Runtime.InteropServices;
using System.Threading;
using System.Threading.Tasks;

namespace RustEtherNetIp
{
    public partial class EtherNetIpClient
    {
        /// <summary>
        /// Establishes connection to a CompactLogix or ControlLogix PLC via EtherNet/IP with route path.
        /// Use this method for ControlLogix systems where the CPU is in a specific slot.
        /// </summary>
        public bool ConnectWithRoute(string address, RoutePath routePath)
        {
            if (_isDisposed)
                throw new ObjectDisposedException(nameof(EtherNetIpClient));

            lock (_lock)
            {
                if (_clientId != -1)
                    throw new InvalidOperationException("Already connected to a PLC. Call Disconnect() first.");

                _ = routePath ?? throw new ArgumentNullException(nameof(routePath));

                bool connected = ConnectNative(address, routePath, startKeepAlive: true);
                return connected;
            }
        }

        /// <summary>
        /// Establishes connection to a CompactLogix or ControlLogix PLC via EtherNet/IP.
        /// </summary>
        public bool Connect(string address)
        {
            if (_isDisposed)
                throw new ObjectDisposedException(nameof(EtherNetIpClient));

            lock (_lock)
            {
                if (_clientId != -1)
                    throw new InvalidOperationException("Already connected to a PLC. Call Disconnect() first.");

                bool connected = ConnectNative(address, null, startKeepAlive: true);
                return connected;
            }
        }

        /// <summary>
        /// Disconnects from the PLC and cleans up the EtherNet/IP session.
        /// </summary>
        public void Disconnect()
        {
            lock (_lock)
            {
                StopKeepAlive();
                _operationLock.Wait();
                try
                {
                    DisconnectNativeLocked(clearAddress: true);
                }
                finally
                {
                    _operationLock.Release();
                }
            }
        }

        public bool IsConnected => _clientId >= 0;

        public ClientStatistics Statistics => _statistics;

        public int ClientId => _clientId;

        private void StartKeepAlive()
        {
            _keepAliveCts?.Cancel();
            _keepAliveCts?.Dispose();
            _keepAliveCts = new CancellationTokenSource();

            _keepAliveTask = Task.Run(async () =>
            {
                while (!_keepAliveCts.Token.IsCancellationRequested)
                {
                    try
                    {
                        await Task.Delay(_keepAliveInterval, _keepAliveCts.Token);
                        RunKeepAliveTick();
                    }
                    catch (OperationCanceledException)
                    {
                        break;
                    }
                    catch (Exception)
                    {
                    }
                }
            }, _keepAliveCts.Token);
        }

        private void StopKeepAlive()
        {
            _keepAliveCts?.Cancel();
            var keepAliveTask = _keepAliveTask;
            if (keepAliveTask != null && Task.CurrentId != keepAliveTask.Id)
            {
                try
                {
                    keepAliveTask.Wait(1000);
                }
                catch (AggregateException ex) when (ex.InnerExceptions.All(inner =>
                    inner is TaskCanceledException || inner is OperationCanceledException))
                {
                }
            }

            _keepAliveTask = null;
        }

        private void RunKeepAliveTick()
        {
            if (!_operationLock.Wait(TimeSpan.FromMilliseconds(100)))
                return;

            try
            {
                if (_clientId < 0 || _isDisposed)
                    return;

                int result = eip_check_health_detailed(_clientId, out int isHealthy);
                if (result == 0 && isHealthy != 0)
                    return;

                var savedAddress = _currentAddress;
                var savedRoute = _currentRoutePath;

                DisconnectNativeLocked(clearAddress: false);
                if (!string.IsNullOrEmpty(savedAddress))
                    ConnectNative(savedAddress, savedRoute, startKeepAlive: false);
            }
            finally
            {
                _operationLock.Release();
            }
        }

        private bool ConnectNative(string address, RoutePath? routePath, bool startKeepAlive)
        {
            int result;
            if (routePath == null)
            {
                IntPtr addressPtr = AllocUtf8(address);
                try
                {
                    result = eip_connect(addressPtr);
                }
                finally
                {
                    FreeUtf8(addressPtr);
                }
            }
            else
            {
                IntPtr addressPtr = AllocUtf8(address);
                try
                {
                    var (hopTypes, ports, slots, addressPtrs, addressHandles) = routePath.PrepareForFFI();
                    try
                    {
                        result = eip_connect_with_route_hops(
                            addressPtr,
                            hopTypes,
                            ports,
                            slots,
                            addressPtrs,
                            hopTypes.Length);
                    }
                    finally
                    {
                        routePath.ReleaseFFIHandles(addressHandles);
                    }
                }
                finally
                {
                    FreeUtf8(addressPtr);
                }
            }

            if (result >= 0)
            {
                _clientId = result;
                _currentAddress = address;
                _currentRoutePath = routePath;
                _lastConnectError = null;
                eip_set_max_packet_size(_clientId, 4000);
                if (startKeepAlive)
                    StartKeepAlive();
                return true;
            }

            _clientId = -1;
            _lastConnectError = DescribeConnectFailure(result, address);
            return false;
        }

        private void DisconnectNativeLocked(bool clearAddress)
        {
            if (_clientId >= 0)
            {
                eip_disconnect(_clientId);
                _clientId = -1;
            }

            if (clearAddress)
            {
                _currentAddress = string.Empty;
                _currentRoutePath = null;
            }

            _tagCache.Clear();
        }

        private static string DescribeConnectFailure(int result, string address)
        {
            return result == EipErrorRuntimeInit
                ? $"Failed to initialize the native EtherNet/IP runtime while connecting to '{address}' (code {EipErrorRuntimeInit})."
                : $"Failed to connect to '{address}' (code {result}).";
        }
    }
}
