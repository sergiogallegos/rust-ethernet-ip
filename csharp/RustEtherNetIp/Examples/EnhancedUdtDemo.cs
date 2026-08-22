using System;

namespace RustEtherNetIp.Examples
{
    /// <summary>
    /// Maintained UDT example using symbolic member paths. Offset-based UDT
    /// member APIs were retired in 1.2.0 and are intentionally not demonstrated.
    /// </summary>
    internal static class EnhancedUdtDemo
    {
        public static void Run(string[] args)
        {
            string address = args.Length > 0
                ? args[0]
                : Environment.GetEnvironmentVariable("PLC_ADDRESS") ?? "192.168.0.10:44818";

            using var client = new EtherNetIpClient();
            if (!client.Connect(address))
                throw new InvalidOperationException(client.LastConnectError);

            // Whole-structure reads automatically use fragmentation when needed.
            PlcValue structure = client.ReadUdtChunked("Mixer");
            Console.WriteLine($"Mixer structure: {structure}");

            // Prefer exact symbolic paths for maintained member access.
            int speed = client.ReadDint("Mixer.CommandSpeed");
            string description = client.ReadString("Mixer.Description");
            Console.WriteLine($"Speed={speed}, Description={description}");

            client.WriteDint("Mixer.CommandSpeed", 1250);
            client.WriteString("Mixer.Description", "Primary mixer");

            // The same typed path works for members of a UDT array element.
            client.WriteReal("Motors[0].SpeedSetpoint", 60.0f);
            client.WriteString("Motors[0].Description", "Infeed conveyor");
        }
    }
}
