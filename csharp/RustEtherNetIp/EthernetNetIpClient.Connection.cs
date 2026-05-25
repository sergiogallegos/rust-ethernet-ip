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

                IntPtr addressPtr = Marshal.StringToHGlobalAnsi(address);
                try
                {
                    var (hopTypes, ports, slots, addressPtrs, addressHandles) = routePath.PrepareForFFI();

                    try
                    {
                        _clientId = eip_connect_with_route_hops(
                            addressPtr,
                            hopTypes,
                            ports,
                            slots,
                            addressPtrs,
                            hopTypes.Length);

                        if (_clientId >= 0)
                        {
                            _currentAddress = address;
                            _currentRoutePath = routePath;
                            eip_set_max_packet_size(_clientId, 4000);
                            StartKeepAlive();
                        }
                    }
                    finally
                    {
                        routePath.ReleaseFFIHandles(addressHandles);
                    }

                    return _clientId >= 0;
                }
                finally
                {
                    Marshal.FreeHGlobal(addressPtr);
                }
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

                IntPtr addressPtr = Marshal.StringToHGlobalAnsi(address);
                try
                {
                    _clientId = eip_connect(addressPtr);
                    if (_clientId >= 0)
                    {
                        _currentAddress = address;
                        eip_set_max_packet_size(_clientId, 4000);
                        StartKeepAlive();
                    }
                    return _clientId >= 0;
                }
                finally
                {
                    Marshal.FreeHGlobal(addressPtr);
                }
            }
        }

        /// <summary>
        /// Disconnects from the PLC and cleans up the EtherNet/IP session.
        /// </summary>
        public void Disconnect()
        {
            lock (_lock)
            {
                if (_clientId >= 0)
                {
                    StopKeepAlive();
                    eip_disconnect(_clientId);
                    _clientId = -1;
                    _currentAddress = string.Empty;
                    _tagCache.Clear();
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
                        await Task.Delay(30000, _keepAliveCts.Token);
                        if (_clientId >= 0)
                        {
                            int isHealthy;
                            if (eip_check_health_detailed(_clientId, out isHealthy) != 0 || isHealthy == 0)
                            {
                                var savedAddress = _currentAddress;
                                var savedRoute = _currentRoutePath;
                                Disconnect();
                                if (!string.IsNullOrEmpty(savedAddress))
                                {
                                    if (savedRoute != null)
                                        ConnectWithRoute(savedAddress, savedRoute);
                                    else
                                        Connect(savedAddress);
                                }
                            }
                        }
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
    }
}
