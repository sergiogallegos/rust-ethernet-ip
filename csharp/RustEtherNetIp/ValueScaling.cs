using System;

namespace RustEtherNetIp
{
    /// <summary>
    /// Provides utility methods for scaling and converting PLC values.
    /// Useful for engineering unit conversion and value transformation.
    /// </summary>
    public static class ValueScaling
    {
        /// <summary>
        /// Conversion modes for value scaling.
        /// </summary>
        public enum ConversionMode
        {
            /// <summary>
            /// No conversion - return value as-is.
            /// </summary>
            None,

            /// <summary>
            /// Linear scaling: output = minScale + (value - minRaw) * (maxScale - minScale) / (maxRaw - minRaw)
            /// </summary>
            Linear,

            /// <summary>
            /// Square root scaling: output = minScale + sqrt((value - minRaw) / (maxRaw - minRaw)) * (maxScale - minScale)
            /// </summary>
            SquareRoot
        }

        /// <summary>
        /// Scales a raw integer value using linear conversion.
        /// </summary>
        /// <param name="rawValue">The raw value from the PLC</param>
        /// <param name="minRaw">Minimum raw value</param>
        /// <param name="maxRaw">Maximum raw value</param>
        /// <param name="minScale">Minimum scaled value</param>
        /// <param name="maxScale">Maximum scaled value</param>
        /// <returns>The scaled value</returns>
        public static double ScaleLinear(int rawValue, double minRaw, double maxRaw, double minScale, double maxScale)
        {
            if (Math.Abs(maxRaw - minRaw) < 0.0001)
                return minScale;

            double ratio = (rawValue - minRaw) / (maxRaw - minRaw);
            return minScale + ratio * (maxScale - minScale);
        }

        /// <summary>
        /// Scales a raw floating-point value using linear conversion.
        /// </summary>
        /// <param name="rawValue">The raw value from the PLC</param>
        /// <param name="minRaw">Minimum raw value</param>
        /// <param name="maxRaw">Maximum raw value</param>
        /// <param name="minScale">Minimum scaled value</param>
        /// <param name="maxScale">Maximum scaled value</param>
        /// <returns>The scaled value</returns>
        public static double ScaleLinear(double rawValue, double minRaw, double maxRaw, double minScale, double maxScale)
        {
            if (Math.Abs(maxRaw - minRaw) < 0.0001)
                return minScale;

            double ratio = (rawValue - minRaw) / (maxRaw - minRaw);
            return minScale + ratio * (maxScale - minScale);
        }

        /// <summary>
        /// Scales a raw value using square root conversion.
        /// Useful for flow measurements where the relationship is non-linear.
        /// </summary>
        /// <param name="rawValue">The raw value from the PLC</param>
        /// <param name="minRaw">Minimum raw value</param>
        /// <param name="maxRaw">Maximum raw value</param>
        /// <param name="minScale">Minimum scaled value</param>
        /// <param name="maxScale">Maximum scaled value</param>
        /// <returns>The scaled value</returns>
        public static double ScaleSquareRoot(int rawValue, double minRaw, double maxRaw, double minScale, double maxScale)
        {
            if (Math.Abs(maxRaw - minRaw) < 0.0001)
                return minScale;

            double ratio = (rawValue - minRaw) / (maxRaw - minRaw);
            if (ratio < 0)
                ratio = 0;
            if (ratio > 1)
                ratio = 1;

            double sqrtRatio = Math.Sqrt(ratio);
            return minScale + sqrtRatio * (maxScale - minScale);
        }

        /// <summary>
        /// Scales a raw floating-point value using square root conversion.
        /// </summary>
        /// <param name="rawValue">The raw value from the PLC</param>
        /// <param name="minRaw">Minimum raw value</param>
        /// <param name="maxRaw">Maximum raw value</param>
        /// <param name="minScale">Minimum scaled value</param>
        /// <param name="maxScale">Maximum scaled value</param>
        /// <returns>The scaled value</returns>
        public static double ScaleSquareRoot(double rawValue, double minRaw, double maxRaw, double minScale, double maxScale)
        {
            if (Math.Abs(maxRaw - minRaw) < 0.0001)
                return minScale;

            double ratio = (rawValue - minRaw) / (maxRaw - minRaw);
            if (ratio < 0)
                ratio = 0;
            if (ratio > 1)
                ratio = 1;

            double sqrtRatio = Math.Sqrt(ratio);
            return minScale + sqrtRatio * (maxScale - minScale);
        }

        /// <summary>
        /// Scales a PlcValue using the specified conversion mode.
        /// </summary>
        /// <param name="value">The PlcValue to scale</param>
        /// <param name="mode">The conversion mode to use</param>
        /// <param name="minRaw">Minimum raw value</param>
        /// <param name="maxRaw">Maximum raw value</param>
        /// <param name="minScale">Minimum scaled value</param>
        /// <param name="maxScale">Maximum scaled value</param>
        /// <returns>The scaled value as a double</returns>
        /// <exception cref="ArgumentException">Thrown if value type is not numeric</exception>
        public static double Scale(PlcValue value, ConversionMode mode, double minRaw, double maxRaw, double minScale, double maxScale)
        {
            double rawValue = value.Type switch
            {
                PlcValueType.Dint => value.As<int>(),
                PlcValueType.Int => value.As<short>(),
                PlcValueType.Real => value.As<float>(),
                PlcValueType.Lreal => value.As<double>(),
                PlcValueType.Sint => value.As<sbyte>(),
                PlcValueType.Uint => value.As<ushort>(),
                PlcValueType.Udint => value.As<uint>(),
                _ => throw new ArgumentException($"Cannot scale non-numeric type: {value.Type}", nameof(value))
            };

            return mode switch
            {
                ConversionMode.None => rawValue,
                ConversionMode.Linear => ScaleLinear(rawValue, minRaw, maxRaw, minScale, maxScale),
                ConversionMode.SquareRoot => ScaleSquareRoot(rawValue, minRaw, maxRaw, minScale, maxScale),
                _ => throw new ArgumentException($"Unknown conversion mode: {mode}", nameof(mode))
            };
        }
    }
}

