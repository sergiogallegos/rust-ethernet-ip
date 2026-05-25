using System;
using System.Reflection;
using System.Runtime.InteropServices;
using Xunit;

namespace RustEtherNetIp.Tests
{
    public class RoutePathContractTests
    {
        [Fact]
        public void EmptyRoutePath_HasNoHops()
        {
            var route = new RoutePath();

            Assert.True(route.IsEmpty);
            Assert.Empty(route.Slots);
            Assert.Empty(route.Ports);
            Assert.Empty(route.Addresses);
        }

        [Fact]
        public void FluentAddMethods_PreserveOrderedHopsAndGroupedViews()
        {
            var route = new RoutePath()
                .AddSlot(0)
                .AddPort(2)
                .AddAddress("192.168.1.10")
                .AddSlot(1);

            Assert.False(route.IsEmpty);
            Assert.Equal(new byte[] { 0, 1 }, route.Slots);
            Assert.Equal(new byte[] { 2 }, route.Ports);
            Assert.Equal(new[] { "192.168.1.10" }, route.Addresses);
            Assert.Equal(3, route.Hops.Count);
        }

        [Theory]
        [InlineData("")]
        [InlineData(null)]
        public void AddAddress_RejectsEmptyAddress(string? address)
        {
            var route = new RoutePath();

            Assert.Throws<ArgumentException>(() => route.AddAddress(address!));
        }

        [Fact]
        public void PrepareForFfi_ProducesPinnedNullTerminatedAddresses()
        {
            var route = new RoutePath()
                .AddSlot(3)
                .AddPort(2)
                .AddAddress("10.20.30.40");

            var prepared = InvokePrepareForFfi(route);
            try
            {
                Assert.Equal(new byte[] { 1, 2 }, prepared.HopTypes);
                Assert.Equal(new byte[] { 1, 2 }, prepared.Ports);
                Assert.Equal(new byte[] { 3, 0 }, prepared.Slots);
                var ptr = prepared.AddressPtrs[1];
                Assert.NotEqual(IntPtr.Zero, ptr);
                Assert.Equal("10.20.30.40", Marshal.PtrToStringAnsi(ptr));
                Assert.Equal(2, prepared.AddressHandles.Length);
                Assert.True(prepared.AddressHandles[1].IsAllocated);
            }
            finally
            {
                InvokeReleaseFfiHandles(route, prepared.AddressHandles);
            }
        }

        private static (byte[] HopTypes, byte[] Ports, byte[] Slots, IntPtr[] AddressPtrs, GCHandle[] AddressHandles)
            InvokePrepareForFfi(RoutePath route)
        {
            var method = typeof(RoutePath).GetMethod("PrepareForFFI", BindingFlags.Instance | BindingFlags.NonPublic)
                ?? throw new MissingMethodException(nameof(RoutePath), "PrepareForFFI");
            return ((byte[] HopTypes, byte[] Ports, byte[] Slots, IntPtr[] AddressPtrs, GCHandle[] AddressHandles))method.Invoke(route, null)!;
        }

        private static void InvokeReleaseFfiHandles(RoutePath route, GCHandle[] handles)
        {
            var method = typeof(RoutePath).GetMethod("ReleaseFFIHandles", BindingFlags.Instance | BindingFlags.NonPublic)
                ?? throw new MissingMethodException(nameof(RoutePath), "ReleaseFFIHandles");
            method.Invoke(route, new object[] { handles });
        }
    }
}
