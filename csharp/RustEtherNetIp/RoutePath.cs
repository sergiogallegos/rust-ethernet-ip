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
        private readonly List<byte> _slots = new List<byte>();
        private readonly List<byte> _ports = new List<byte>();
        private readonly List<string> _addresses = new List<string>();

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
            _slots.Add(slot);
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
            _ports.Add(port);
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
            
            _addresses.Add(address);
            return this;
        }

        /// <summary>
        /// Gets the slots in this route path
        /// </summary>
        public IReadOnlyList<byte> Slots => _slots;

        /// <summary>
        /// Gets the ports in this route path
        /// </summary>
        public IReadOnlyList<byte> Ports => _ports;

        /// <summary>
        /// Gets the addresses in this route path
        /// </summary>
        public IReadOnlyList<string> Addresses => _addresses;

        /// <summary>
        /// Checks if this route path is empty (no slots, ports, or addresses)
        /// </summary>
        public bool IsEmpty => _slots.Count == 0 && _ports.Count == 0 && _addresses.Count == 0;

        internal (byte[] slots, byte[] ports, IntPtr[] addressPtrs, GCHandle[] addressHandles) PrepareForFFI()
        {
            // Prepare slots
            byte[] slots = _slots.ToArray();

            // Prepare ports
            byte[] ports = _ports.ToArray();

            // Prepare addresses
            IntPtr[] addressPtrs = new IntPtr[_addresses.Count];
            GCHandle[] addressHandles = new GCHandle[_addresses.Count];
            
            for (int i = 0; i < _addresses.Count; i++)
            {
                byte[] addressBytes = System.Text.Encoding.UTF8.GetBytes(_addresses[i] + "\0");
                GCHandle handle = GCHandle.Alloc(addressBytes, GCHandleType.Pinned);
                addressHandles[i] = handle;
                addressPtrs[i] = handle.AddrOfPinnedObject();
            }

            return (slots, ports, addressPtrs, addressHandles);
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
}

