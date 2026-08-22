using System;

namespace RustEtherNetIp.Examples
{
    /// <summary>
    /// Demonstrates nested UDT access through full Logix symbolic paths.
    /// The public type and entry point remain for source compatibility.
    /// </summary>
    public class NestedUdtExample
    {
        public static void RunExample()
        {
            string address = Environment.GetEnvironmentVariable("PLC_ADDRESS")
                ?? "192.168.0.10:44818";

            using var client = new EtherNetIpClient();
            if (!client.Connect(address))
                throw new InvalidOperationException(client.LastConnectError);

            bool running = client.ReadBool("ProductionLine.Station1.Motor.Running");
            float speed = client.ReadReal("ProductionLine.Station1.Motor.Speed");
            string status = client.ReadString("ProductionLine.Station1.Status");

            Console.WriteLine($"Running={running}, Speed={speed}, Status={status}");

            client.WriteReal("ProductionLine.Station1.Motor.Speed", 1500.0f);
            client.WriteString("ProductionLine.Station1.Status", "Active");

            // Whole nested structures can be read when the application needs raw
            // structure data. Member-level writes are the preferred update path.
            PlcValue line = client.ReadUdtChunked("ProductionLine");
            Console.WriteLine(line);
        }
    }
}
