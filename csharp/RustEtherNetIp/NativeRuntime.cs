using System;
using System.Runtime.InteropServices;

namespace RustEtherNetIp
{
    public static class NativeRuntime
    {
        public const int ExpectedAbiVersion = 3;

        static NativeRuntime()
        {
            AbiVersion = eip_abi_version();
            if (AbiVersion != ExpectedAbiVersion)
            {
                throw new BadImageFormatException(
                    $"Native rust_ethernet_ip ABI version {AbiVersion}, wrapper expects {ExpectedAbiVersion}.");
            }

            LibraryVersion = Marshal.PtrToStringUTF8(eip_library_version()) ?? string.Empty;
            Capabilities = eip_capabilities();
        }

        public static uint AbiVersion { get; }

        public static string LibraryVersion { get; }

        public static ulong Capabilities { get; }

        [DllImport("rust_ethernet_ip", CallingConvention = CallingConvention.Cdecl)]
        private static extern uint eip_abi_version();

        [DllImport("rust_ethernet_ip", CallingConvention = CallingConvention.Cdecl)]
        private static extern IntPtr eip_library_version();

        [DllImport("rust_ethernet_ip", CallingConvention = CallingConvention.Cdecl)]
        private static extern ulong eip_capabilities();
    }
}
