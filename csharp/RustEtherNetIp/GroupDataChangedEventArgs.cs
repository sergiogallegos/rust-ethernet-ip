using System;
using System.Collections.Generic;

namespace RustEtherNetIp
{
    /// <summary>
    /// Event arguments for TagGroup data changed events.
    /// Contains information about which tags changed and their new values.
    /// </summary>
    public class GroupDataChangedEventArgs : EventArgs
    {
        /// <summary>
        /// Gets the array of tag names that changed in this update cycle.
        /// </summary>
        public string[] ChangedTags { get; set; } = Array.Empty<string>();

        /// <summary>
        /// Gets a dictionary of all tag names and their current values.
        /// </summary>
        public Dictionary<string, PlcValue> AllValues { get; set; } = new();
    }
}

