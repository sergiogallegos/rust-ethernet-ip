using System;

namespace RustEtherNetIp
{
    /// <summary>
    /// Represents a subscription to a PLC tag for real-time updates.
    /// </summary>
    public class TagSubscription
    {
        /// <summary>
        /// The name of the subscribed tag
        /// </summary>
        public string TagName { get; }

        /// <summary>
        /// The current value of the tag
        /// </summary>
        public object? Value { get; private set; }

        /// <summary>
        /// The last time the value was updated
        /// </summary>
        public DateTime LastUpdate { get; private set; }

        /// <summary>
        /// Event raised when the tag value changes
        /// </summary>
        public event EventHandler<TagValueChangedEventArgs>? ValueChanged;

        internal TagSubscription(string tagName)
        {
            TagName = tagName;
            LastUpdate = DateTime.MinValue;
            ValueChanged = null; // Initialize to null
        }

        internal void UpdateValue(object? newValue)
        {
            if (!Equals(Value, newValue))
            {
                var oldValue = Value;
                Value = newValue;
                LastUpdate = DateTime.Now;
                ValueChanged?.Invoke(this, new TagValueChangedEventArgs(TagName, oldValue, newValue));
            }
        }
    }

    /// <summary>
    /// Event arguments for tag value changes
    /// </summary>
    public class TagValueChangedEventArgs : EventArgs
    {
        /// <summary>
        /// The name of the tag that changed
        /// </summary>
        public string TagName { get; }

        /// <summary>
        /// The previous value of the tag
        /// </summary>
        public object? OldValue { get; }

        /// <summary>
        /// The new value of the tag
        /// </summary>
        public object? NewValue { get; }

        public TagValueChangedEventArgs(string tagName, object? oldValue, object? newValue)
        {
            TagName = tagName;
            OldValue = oldValue;
            NewValue = newValue;
        }
    }
} 