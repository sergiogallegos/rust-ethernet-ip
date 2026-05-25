using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;

namespace RustEtherNetIp
{
    /// <summary>
    /// Represents a route path for PLC communication, used for ControlLogix backplane routing
    /// and multi-hop network routing.
    /// </summary>
    /// <remarks>
    /// Route paths are essential for ControlLogix systems where the CPU is in a specific slot.
    /// For CompactLogix (built-in Ethernet), route paths are typically not needed.
    /// 
    /// Example usage:
    /// <code>
    /// // ControlLogix: CPU in Slot 0
    /// var route = new RoutePath().AddSlot(0);
    /// var client = new EtherNetIpClient();
    /// client.ConnectWithRoute("192.168.0.1:44818", route);
    /// 
    /// // ControlLogix: CPU in Slot 3
    /// var route3 = new RoutePath().AddSlot(3);
    /// client.ConnectWithRoute("192.168.0.1:44818", route3);
    /// 
    /// // Network routing (multi-hop)
    /// var networkRoute = new RoutePath()
    ///     .AddPort(2)  // Port 2 = Ethernet
    ///     .AddAddress("192.168.1.100")  // Remote Ethernet module IP
    ///     .AddSlot(0);  // CPU slot on remote PLC
    /// </code>
    /// </remarks>
    public class RoutePath
    {
        private readonly List<RouteHop> _hops = new List<RouteHop>();

        /// <summary>
        /// Creates a new empty route path
        /// </summary>
        public RoutePath()
        {
        }

        /// <summary>
        /// Adds a backplane slot to the route path
        /// </summary>
        /// <param name="slot">The slot number (0-255) where the CPU is located</param>
        /// <returns>This RoutePath instance for method chaining</returns>
        /// <example>
        /// <code>
        /// var route = new RoutePath().AddSlot(0);  // CPU in Slot 0
        /// </code>
        /// </example>
        public RoutePath AddSlot(byte slot)
        {
            return AddBackplane(1, slot);
        }

        public RoutePath AddBackplane(byte port, byte slot)
        {
            _hops.Add(RouteHop.Backplane(port, slot));
            return this;
        }

        /// <summary>
        /// Adds a network port to the route path
        /// </summary>
        /// <param name="port">The port number (typically 2 for Ethernet)</param>
        /// <returns>This RoutePath instance for method chaining</returns>
        /// <example>
        /// <code>
        /// var route = new RoutePath().AddPort(2);  // Port 2 = Ethernet
        /// </code>
        /// </example>
        public RoutePath AddPort(byte port)
        {
            for (int i = _hops.Count - 1; i >= 0; i--)
            {
                if (_hops[i].Kind == RouteHopKind.Ethernet)
                {
                    _hops[i] = RouteHop.Ethernet(port, _hops[i].Address!);
                    break;
                }
            }
            return this;
        }

        /// <summary>
        /// Adds a network address to the route path (for multi-hop routing)
        /// </summary>
        /// <param name="address">The IP address or network address</param>
        /// <returns>This RoutePath instance for method chaining</returns>
        /// <example>
        /// <code>
        /// var route = new RoutePath()
        ///     .AddPort(2)
        ///     .AddAddress("192.168.1.100")
        ///     .AddSlot(0);
        /// </code>
        /// </example>
        public RoutePath AddAddress(string address)
        {
            if (string.IsNullOrEmpty(address))
                throw new ArgumentException("Address cannot be null or empty", nameof(address));
            
            return AddEthernet(2, address);
        }

        public RoutePath AddEthernet(byte port, string address)
        {
            if (string.IsNullOrEmpty(address))
                throw new ArgumentException("Address cannot be null or empty", nameof(address));

            _hops.Add(RouteHop.Ethernet(port, address));
            return this;
        }

        /// <summary>
        /// Gets the slots in this route path
        /// </summary>
        public IReadOnlyList<byte> Slots
        {
            get
            {
                var slots = new List<byte>();
                foreach (var hop in _hops)
                {
                    if (hop.Kind == RouteHopKind.Backplane)
                        slots.Add(hop.Slot);
                }
                return slots;
            }
        }

        /// <summary>
        /// Gets the ports in this route path
        /// </summary>
        public IReadOnlyList<byte> Ports
        {
            get
            {
                var ports = new List<byte>();
                foreach (var hop in _hops)
                {
                    if (hop.Kind == RouteHopKind.Ethernet)
                        ports.Add(hop.Port);
                }
                return ports;
            }
        }

        /// <summary>
        /// Gets the addresses in this route path
        /// </summary>
        public IReadOnlyList<string> Addresses
        {
            get
            {
                var addresses = new List<string>();
                foreach (var hop in _hops)
                {
                    if (hop.Kind == RouteHopKind.Ethernet && hop.Address != null)
                        addresses.Add(hop.Address);
                }
                return addresses;
            }
        }

        public IReadOnlyList<RouteHop> Hops => _hops;

        /// <summary>
        /// Checks if this route path is empty (no slots, ports, or addresses)
        /// </summary>
        public bool IsEmpty => _hops.Count == 0;

        internal (byte[] hopTypes, byte[] ports, byte[] slots, IntPtr[] addressPtrs, GCHandle[] addressHandles) PrepareForFFI()
        {
            byte[] hopTypes = new byte[_hops.Count];
            byte[] ports = new byte[_hops.Count];
            byte[] slots = new byte[_hops.Count];
            IntPtr[] addressPtrs = new IntPtr[_hops.Count];
            GCHandle[] addressHandles = new GCHandle[_hops.Count];
            
            for (int i = 0; i < _hops.Count; i++)
            {
                var hop = _hops[i];
                hopTypes[i] = hop.Kind == RouteHopKind.Backplane ? (byte)1 : (byte)2;
                ports[i] = hop.Port;
                slots[i] = hop.Slot;
                if (hop.Kind == RouteHopKind.Ethernet)
                {
                    byte[] addressBytes = System.Text.Encoding.UTF8.GetBytes(hop.Address! + "\0");
                    GCHandle handle = GCHandle.Alloc(addressBytes, GCHandleType.Pinned);
                    addressHandles[i] = handle;
                    addressPtrs[i] = handle.AddrOfPinnedObject();
                }
            }

            return (hopTypes, ports, slots, addressPtrs, addressHandles);
        }

        internal void ReleaseFFIHandles(GCHandle[] handles)
        {
            foreach (var handle in handles)
            {
                if (handle.IsAllocated)
                {
                    handle.Free();
                }
            }
        }
    }

    public enum RouteHopKind
    {
        Backplane = 1,
        Ethernet = 2
    }

    public readonly struct RouteHop
    {
        private RouteHop(RouteHopKind kind, byte port, byte slot, string? address)
        {
            Kind = kind;
            Port = port;
            Slot = slot;
            Address = address;
        }

        public RouteHopKind Kind { get; }
        public byte Port { get; }
        public byte Slot { get; }
        public string? Address { get; }

        public static RouteHop Backplane(byte port, byte slot) => new RouteHop(RouteHopKind.Backplane, port, slot, null);

        public static RouteHop Ethernet(byte port, string address) => new RouteHop(RouteHopKind.Ethernet, port, 0, address);
    }
}
