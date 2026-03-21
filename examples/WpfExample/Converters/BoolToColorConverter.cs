using System;
using System.Globalization;
using System.Windows.Data;
using System.Windows.Media;

namespace WpfExample.Converters
{
    public class BoolToColorConverter : IValueConverter
    {
        public object Convert(object value, Type targetType, object parameter, CultureInfo culture)
        {
            if (value is bool isConnected)
            {
                return isConnected ? Colors.Green : Colors.Red;
            }
            return Colors.Gray;
        }

        public object ConvertBack(object value, Type targetType, object parameter, CultureInfo culture)
        {
            // One-way UI converter: avoid runtime crash if binding mode changes.
            return Binding.DoNothing;
        }
    }
}
