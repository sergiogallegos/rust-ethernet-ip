using System;
using System.Runtime.InteropServices;

namespace RustEtherNetIp
{
    public partial class EtherNetIpClient
    {
        internal const int EipErrorRuntimeInit = -2;

        // These are the low-level FFI calls to the Rust library.
        // Public APIs should route through the wrapper methods instead.

        [DllImport("rust_ethernet_ip", CallingConvention = CallingConvention.Cdecl)]
        private static extern int eip_connect(IntPtr address);

        [DllImport("rust_ethernet_ip", CallingConvention = CallingConvention.Cdecl)]
        private static extern int eip_connect_with_route(
            IntPtr address,
            byte[] slots,
            int slot_count,
            byte[] ports,
            int port_count,
            IntPtr[] addresses,
            int address_count);

        [DllImport("rust_ethernet_ip", CallingConvention = CallingConvention.Cdecl)]
        private static extern int eip_connect_with_route_hops(
            IntPtr address,
            byte[] hop_types,
            byte[] ports,
            byte[] slots,
            IntPtr[] addresses,
            int hop_count);

        [DllImport("rust_ethernet_ip", CallingConvention = CallingConvention.Cdecl)]
        private static extern int eip_set_route_path(
            int client_id,
            byte[] slots,
            int slot_count,
            byte[] ports,
            int port_count,
            IntPtr[] addresses,
            int address_count);

        [DllImport("rust_ethernet_ip", CallingConvention = CallingConvention.Cdecl)]
        private static extern int eip_set_route_path_hops(
            int client_id,
            byte[] hop_types,
            byte[] ports,
            byte[] slots,
            IntPtr[] addresses,
            int hop_count);

        [DllImport("rust_ethernet_ip", CallingConvention = CallingConvention.Cdecl)]
        private static extern int eip_disconnect(int client_id);

        [DllImport("rust_ethernet_ip", CallingConvention = CallingConvention.Cdecl)]
        private static extern int eip_read_bool(int client_id, IntPtr tag_name, out int result);

        [DllImport("rust_ethernet_ip", CallingConvention = CallingConvention.Cdecl)]
        private static extern int eip_write_bool(int client_id, IntPtr tag_name, int value);

        [DllImport("rust_ethernet_ip", CallingConvention = CallingConvention.Cdecl)]
        private static extern int eip_read_sint(int client_id, IntPtr tag_name, out sbyte result);

        [DllImport("rust_ethernet_ip", CallingConvention = CallingConvention.Cdecl)]
        private static extern int eip_write_sint(int client_id, IntPtr tag_name, sbyte value);

        [DllImport("rust_ethernet_ip", CallingConvention = CallingConvention.Cdecl)]
        private static extern int eip_read_int(int client_id, IntPtr tag_name, out short result);

        [DllImport("rust_ethernet_ip", CallingConvention = CallingConvention.Cdecl)]
        private static extern int eip_write_int(int client_id, IntPtr tag_name, short value);

        [DllImport("rust_ethernet_ip", CallingConvention = CallingConvention.Cdecl)]
        private static extern int eip_read_dint(int client_id, IntPtr tag_name, out int result);

        [DllImport("rust_ethernet_ip", CallingConvention = CallingConvention.Cdecl)]
        private static extern int eip_write_dint(int client_id, IntPtr tag_name, int value);

        [DllImport("rust_ethernet_ip", CallingConvention = CallingConvention.Cdecl)]
        private static extern int eip_read_lint(int client_id, IntPtr tag_name, out long result);

        [DllImport("rust_ethernet_ip", CallingConvention = CallingConvention.Cdecl)]
        private static extern int eip_write_lint(int client_id, IntPtr tag_name, long value);

        [DllImport("rust_ethernet_ip", CallingConvention = CallingConvention.Cdecl)]
        private static extern int eip_read_usint(int client_id, IntPtr tag_name, out byte result);

        [DllImport("rust_ethernet_ip", CallingConvention = CallingConvention.Cdecl)]
        private static extern int eip_write_usint(int client_id, IntPtr tag_name, byte value);

        [DllImport("rust_ethernet_ip", CallingConvention = CallingConvention.Cdecl)]
        private static extern int eip_read_uint(int client_id, IntPtr tag_name, out ushort result);

        [DllImport("rust_ethernet_ip", CallingConvention = CallingConvention.Cdecl)]
        private static extern int eip_write_uint(int client_id, IntPtr tag_name, ushort value);

        [DllImport("rust_ethernet_ip", CallingConvention = CallingConvention.Cdecl)]
        private static extern int eip_read_udint(int client_id, IntPtr tag_name, out uint result);

        [DllImport("rust_ethernet_ip", CallingConvention = CallingConvention.Cdecl)]
        private static extern int eip_write_udint(int client_id, IntPtr tag_name, uint value);

        [DllImport("rust_ethernet_ip", CallingConvention = CallingConvention.Cdecl)]
        private static extern int eip_read_ulint(int client_id, IntPtr tag_name, out ulong result);

        [DllImport("rust_ethernet_ip", CallingConvention = CallingConvention.Cdecl)]
        private static extern int eip_write_ulint(int client_id, IntPtr tag_name, ulong value);

        [DllImport("rust_ethernet_ip", CallingConvention = CallingConvention.Cdecl)]
        private static extern int eip_read_real(int client_id, IntPtr tag_name, out double result);

        [DllImport("rust_ethernet_ip", CallingConvention = CallingConvention.Cdecl)]
        private static extern int eip_write_real(int client_id, IntPtr tag_name, double value);

        [DllImport("rust_ethernet_ip", CallingConvention = CallingConvention.Cdecl)]
        private static extern int eip_read_lreal(int client_id, IntPtr tag_name, out double result);

        [DllImport("rust_ethernet_ip", CallingConvention = CallingConvention.Cdecl)]
        private static extern int eip_write_lreal(int client_id, IntPtr tag_name, double value);

        [DllImport("rust_ethernet_ip", CallingConvention = CallingConvention.Cdecl)]
        private static extern int eip_read_string(int client_id, IntPtr tag_name, IntPtr result, int max_length);

        [DllImport("rust_ethernet_ip", CallingConvention = CallingConvention.Cdecl)]
        private static extern int eip_write_string(int client_id, IntPtr tag_name, IntPtr value);

        [DllImport("rust_ethernet_ip", CallingConvention = CallingConvention.Cdecl)]
        private static extern int eip_read_tag(int client_id, IntPtr tag_name, IntPtr result, int max_size);

        [DllImport("rust_ethernet_ip", CallingConvention = CallingConvention.Cdecl)]
        private static extern int eip_read_array_range(
            int client_id,
            IntPtr base_array_name,
            int start_index,
            int element_count,
            IntPtr result,
            int max_size);

        [DllImport("rust_ethernet_ip", CallingConvention = CallingConvention.Cdecl)]
        private static extern int eip_read_udt(int client_id, IntPtr tag_name, IntPtr result, int max_size);

        [DllImport("rust_ethernet_ip", CallingConvention = CallingConvention.Cdecl)]
        private static extern int eip_write_udt(int client_id, IntPtr tag_name, IntPtr value, int size);

        [DllImport("rust_ethernet_ip", CallingConvention = CallingConvention.Cdecl)]
        private static extern int eip_read_udt_chunked(int client_id, IntPtr tag_name, IntPtr result, int max_size);

        [DllImport("rust_ethernet_ip", CallingConvention = CallingConvention.Cdecl)]
        private static extern int eip_read_udt_member_by_offset(int client_id, IntPtr udt_name, int member_offset, int member_size, short data_type, IntPtr result, int max_size);

        [DllImport("rust_ethernet_ip", CallingConvention = CallingConvention.Cdecl)]
        private static extern int eip_write_udt_member_by_offset(int client_id, IntPtr udt_name, int member_offset, int member_size, short data_type, IntPtr value, int size);

        [DllImport("rust_ethernet_ip", CallingConvention = CallingConvention.Cdecl)]
        private static extern int eip_discover_tags(int client_id);

        [DllImport("rust_ethernet_ip", CallingConvention = CallingConvention.Cdecl)]
        private static extern int eip_get_tag_metadata(int client_id, IntPtr tag_name, out TagMetadata metadata);

        [DllImport("rust_ethernet_ip", CallingConvention = CallingConvention.Cdecl)]
        private static extern int eip_get_udt_definition_by_id(int client_id, IntPtr udt_name, IntPtr result_ptr);

        [DllImport("rust_ethernet_ip", CallingConvention = CallingConvention.Cdecl)]
        private static extern int eip_get_tag_attributes_by_id(int client_id, IntPtr tag_name, IntPtr result_ptr);

        [DllImport("rust_ethernet_ip", CallingConvention = CallingConvention.Cdecl)]
        private static extern int eip_discover_tags_detailed_by_id(int client_id, IntPtr result_ptr);

        [DllImport("rust_ethernet_ip", CallingConvention = CallingConvention.Cdecl)]
        private static extern void eip_free_udt_definition(IntPtr result_ptr);

        [DllImport("rust_ethernet_ip", CallingConvention = CallingConvention.Cdecl)]
        private static extern void eip_free_tag_attributes_result(IntPtr result_ptr);

        [DllImport("rust_ethernet_ip", CallingConvention = CallingConvention.Cdecl)]
        private static extern void eip_free_tag_discovery_result(IntPtr result_ptr);

        [DllImport("rust_ethernet_ip", CallingConvention = CallingConvention.Cdecl)]
        private static extern int eip_set_max_packet_size(int client_id, int size);

        [DllImport("rust_ethernet_ip", CallingConvention = CallingConvention.Cdecl)]
        private static extern int eip_check_health(int client_id, out int is_healthy);

        [DllImport("rust_ethernet_ip", CallingConvention = CallingConvention.Cdecl)]
        private static extern int eip_check_health_detailed(int client_id, out int is_healthy);

        [DllImport("rust_ethernet_ip", CallingConvention = CallingConvention.Cdecl)]
        private static extern int eip_get_diagnostics_json(int client_id, int detailed, out IntPtr result_ptr);

        [DllImport("rust_ethernet_ip", CallingConvention = CallingConvention.Cdecl)]
        private static extern void eip_free_string(IntPtr ptr);

        [DllImport("rust_ethernet_ip", CallingConvention = CallingConvention.Cdecl)]
        private static extern int eip_read_tags_batch(int client_id, IntPtr[] tag_names, int tag_count, IntPtr results, int results_capacity);

        [DllImport("rust_ethernet_ip", CallingConvention = CallingConvention.Cdecl)]
        private static extern int eip_write_tags_batch(int client_id, IntPtr tag_values, int tag_count, IntPtr results, int results_capacity);

        [DllImport("rust_ethernet_ip", CallingConvention = CallingConvention.Cdecl)]
        private static extern int eip_execute_batch(int client_id, IntPtr operations, int operation_count, IntPtr results, int results_capacity);

        [DllImport("rust_ethernet_ip", CallingConvention = CallingConvention.Cdecl)]
        private static extern int eip_configure_batch_operations(int client_id, ref BatchConfigNative config);

        [DllImport("rust_ethernet_ip", CallingConvention = CallingConvention.Cdecl)]
        private static extern int eip_get_batch_config(int client_id, out BatchConfigNative config);
    }
}
