using System;

namespace RustEtherNetIp
{
    /// <summary>
    /// Detailed tag attributes returned by the PLC.
    /// </summary>
    public class TagAttributes
    {
        public string Name { get; set; } = string.Empty;
        public string DataTypeName { get; set; } = string.Empty;
        public short DataType { get; set; }
        public int Size { get; set; }
        public int TemplateInstanceId { get; set; }

        public override string ToString()
        {
            return $"{Name} ({DataTypeName}, 0x{DataType:X4}, size={Size}, template={TemplateInstanceId})";
        }
    }
}
