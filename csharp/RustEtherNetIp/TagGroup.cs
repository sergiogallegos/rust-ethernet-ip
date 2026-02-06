using System;
using System.Collections.Generic;
using System.Timers;

namespace RustEtherNetIp
{
    /// <summary>
    /// Represents a group of tags that can be polled periodically.
    /// Useful for HMI applications that need to update multiple tags regularly.
    /// Automatically polls tags at the specified update rate and fires events when data changes.
    /// </summary>
    /// <remarks>
    /// TagGroup provides automatic periodic polling of multiple tags, eliminating the need
    /// for manual timers in application code. When active, it polls all tags in the group
    /// at the specified interval and fires DataChanged events when any tag value changes.
    /// 
    /// This is particularly useful for:
    /// - HMI applications displaying multiple values
    /// - Dashboards that need regular updates
    /// - Applications monitoring multiple process variables
    /// </remarks>
    /// <example>
    /// <code>
    /// // Create a tag group
    /// var group = new TagGroup(client)
    /// {
    ///     TagNames = new[] { "Temperature", "Pressure", "FlowRate" },
    ///     UpdateRateMs = 500  // Poll every 500ms
    /// };
    /// 
    /// // Optionally disable specific tags
    /// group.SetTagActive("FlowRate", false);  // Skip this tag in scans
    /// 
    /// // Subscribe to data changes
    /// group.DataChanged += (sender, e) =>
    /// {
    ///     foreach (var tagName in e.ChangedTags)
    ///     {
    ///         var value = e.AllValues[tagName];
    ///         Console.WriteLine($"{tagName} = {value}");
    ///     }
    /// };
    /// 
    /// // Start polling (explicit control)
    /// group.Start();
    /// 
    /// // Temporarily suspend polling
    /// group.Suspend();
    /// // ... do something ...
    /// group.Resume();
    /// 
    /// // Later, stop polling
    /// group.Stop();
    /// group.Dispose();
    /// </code>
    /// </example>
    public class TagGroup : IDisposable
    {
        private readonly EtherNetIpClient _client;
        private readonly System.Timers.Timer _timer;
        private Dictionary<string, PlcValue> _lastValues = new();
        private Dictionary<string, bool> _tagActiveStates = new();
        private bool _isActive = false;
        private bool _isSuspended = false;
        private bool _isDisposed = false;
        private bool _isScanning = false; // Prevent overlapping scans
        private int _updateRateMs = 500;

        /// <summary>
        /// Gets or sets the array of tag names to poll.
        /// </summary>
        public string[] TagNames { get; set; } = Array.Empty<string>();

        /// <summary>
        /// Gets or sets the update rate in milliseconds.
        /// The group will poll all tags at this interval when active.
        /// </summary>
        public int UpdateRateMs
        {
            get => _updateRateMs;
            set
            {
                if (value <= 0)
                    throw new ArgumentException("Update rate must be greater than zero", nameof(value));
                
                _updateRateMs = value;
                _timer.Interval = value;
            }
        }

        /// <summary>
        /// Gets or sets whether the group is actively polling.
        /// When set to true, polling begins immediately.
        /// When set to false, polling stops.
        /// </summary>
        public bool IsActive
        {
            get => _isActive;
            set
            {
                if (_isDisposed)
                    throw new ObjectDisposedException(nameof(TagGroup));
                
                _isActive = value;
                _timer.Enabled = value;
            }
        }

        /// <summary>
        /// Event fired when one or more tag values change.
        /// The event args contain the list of changed tags and all current values.
        /// </summary>
        public event EventHandler<GroupDataChangedEventArgs>? DataChanged;

        /// <summary>
        /// Gets the time taken for the last scan operation.
        /// </summary>
        public TimeSpan LastScanTime { get; private set; }

        /// <summary>
        /// Gets whether the group is currently suspended (paused but not stopped).
        /// </summary>
        public bool IsSuspended => _isSuspended;

        /// <summary>
        /// Gets or sets whether a specific tag is active in the group scan.
        /// Inactive tags are skipped during polling but remain in the group.
        /// </summary>
        /// <param name="tagName">The tag name to check or set</param>
        /// <returns>True if the tag is active, false if inactive</returns>
        public bool IsTagActive(string tagName)
        {
            return _tagActiveStates.TryGetValue(tagName, out bool active) ? active : true; // Default to active
        }

        /// <summary>
        /// Sets whether a specific tag is active in the group scan.
        /// </summary>
        /// <param name="tagName">The tag name to set</param>
        /// <param name="active">True to enable the tag in scans, false to disable</param>
        public void SetTagActive(string tagName, bool active)
        {
            var tagNames = TagNames ?? Array.Empty<string>();
            if (tagNames.Length == 0 || !Array.Exists(tagNames, t => t == tagName))
                throw new ArgumentException($"Tag '{tagName}' is not in this group", nameof(tagName));
            
            _tagActiveStates[tagName] = active;
        }

        /// <summary>
        /// Initializes a new instance of the TagGroup class.
        /// </summary>
        /// <param name="client">The EtherNet/IP client to use for reading tags</param>
        /// <exception cref="ArgumentNullException">Thrown if client is null</exception>
        public TagGroup(EtherNetIpClient client)
        {
            _client = client ?? throw new ArgumentNullException(nameof(client));
            _timer = new System.Timers.Timer(500); // Default 500ms
            _timer.Elapsed += OnTimerElapsed;
            _updateRateMs = (int)_timer.Interval;
        }

        private void OnTimerElapsed(object? sender, ElapsedEventArgs e)
        {
            var tagNames = TagNames ?? Array.Empty<string>();
            if (_isDisposed || _isSuspended || tagNames.Length == 0)
                return;

            // Prevent overlapping scans - if a scan is still running, skip this one
            if (_isScanning)
            {
                System.Diagnostics.Debug.WriteLine("TagGroup: Skipping scan - previous scan still in progress");
                return;
            }

            _isScanning = true;
            var startTime = DateTime.Now;
            try
            {
                // Filter to only active tags
                var activeTagNames = new List<string>();
                var activeIndices = new List<int>();
                
                for (int i = 0; i < tagNames.Length; i++)
                {
                    var tagName = tagNames[i];
                    if (IsTagActive(tagName))
                    {
                        activeTagNames.Add(tagName);
                        activeIndices.Add(i);
                    }
                }

                if (activeTagNames.Count == 0)
                {
                    System.Diagnostics.Debug.WriteLine("TagGroup: No active tags to read");
                    _isScanning = false;
                    return;
                }

                // Read only active tags in batch
                System.Diagnostics.Debug.WriteLine($"TagGroup: Reading {activeTagNames.Count} tags: {string.Join(", ", activeTagNames)}");
                var values = _client.ReadTags(activeTagNames.ToArray());
                System.Diagnostics.Debug.WriteLine($"TagGroup: Read {values.Length} values successfully");
                var changedTags = new List<string>();

                // Compare with last values and update
                for (int i = 0; i < activeTagNames.Count; i++)
                {
                    var tagName = activeTagNames[i];
                    var newValue = values[i];

                    if (!_lastValues.ContainsKey(tagName) || 
                        !AreValuesEqual(_lastValues[tagName], newValue))
                    {
                        _lastValues[tagName] = newValue;
                        changedTags.Add(tagName);
                    }
                    else
                    {
                        // Update the value even if it hasn't changed (for timestamp updates)
                        _lastValues[tagName] = newValue;
                    }
                }

                // Always fire event with all current values (not just when changed)
                // This ensures the UI table is always populated and updated
                var allValues = new Dictionary<string, PlcValue>(_lastValues);
                DataChanged?.Invoke(this, new GroupDataChangedEventArgs
                {
                    ChangedTags = changedTags.ToArray(),
                    AllValues = allValues
                });

                LastScanTime = DateTime.Now - startTime;
            }
            catch (Exception ex)
            {
                // Log error but still fire event with current values (if any) so UI doesn't freeze
                System.Diagnostics.Debug.WriteLine($"TagGroup polling error: {ex.Message}");
                
                // Fire event with existing values even on error, so UI shows something
                // This prevents the table from being empty when there's a read error
                if (_lastValues.Count > 0)
                {
                    var allValues = new Dictionary<string, PlcValue>(_lastValues);
                    DataChanged?.Invoke(this, new GroupDataChangedEventArgs
                    {
                        ChangedTags = Array.Empty<string>(), // No changes on error
                        AllValues = allValues
                    });
                }
                
                LastScanTime = DateTime.Now - startTime;
            }
            finally
            {
                _isScanning = false;
            }
        }

        private bool AreValuesEqual(PlcValue v1, PlcValue v2)
        {
            if (v1.Type != v2.Type)
                return false;

            return v1.Type switch
            {
                PlcValueType.Bool => v1.As<bool>() == v2.As<bool>(),
                PlcValueType.Dint => v1.As<int>() == v2.As<int>(),
                PlcValueType.Real => Math.Abs(v1.As<float>() - v2.As<float>()) < 0.0001f,
                PlcValueType.String => v1.As<string>() == v2.As<string>(),
                _ => v1.Value?.Equals(v2.Value) ?? false
            };
        }

        /// <summary>
        /// Starts polling the tag group.
        /// Equivalent to setting IsActive = true, but more explicit.
        /// </summary>
        public void Start()
        {
            if (_isDisposed)
                throw new ObjectDisposedException(nameof(TagGroup));
            
            _isSuspended = false;
            _isActive = true;
            _timer.Enabled = true;
        }

        /// <summary>
        /// Stops polling the tag group.
        /// Equivalent to setting IsActive = false, but more explicit.
        /// </summary>
        public void Stop()
        {
            if (_isDisposed)
                throw new ObjectDisposedException(nameof(TagGroup));
            
            _isActive = false;
            _isSuspended = false;
            _timer.Enabled = false;
        }

        /// <summary>
        /// Suspends polling temporarily without stopping.
        /// Use Resume() to continue polling from where it was suspended.
        /// </summary>
        public void Suspend()
        {
            if (_isDisposed)
                throw new ObjectDisposedException(nameof(TagGroup));
            
            if (_isActive)
            {
                _isSuspended = true;
                _timer.Enabled = false;
            }
        }

        /// <summary>
        /// Resumes polling after being suspended.
        /// Only works if the group was previously suspended (not stopped).
        /// </summary>
        public void Resume()
        {
            if (_isDisposed)
                throw new ObjectDisposedException(nameof(TagGroup));
            
            if (_isSuspended && _isActive)
            {
                _isSuspended = false;
                _timer.Enabled = true;
            }
        }

        /// <summary>
        /// Disposes the TagGroup and stops polling.
        /// </summary>
        public void Dispose()
        {
            if (_isDisposed)
                return;

            _isDisposed = true;
            _isActive = false;
            _isSuspended = false;
            _timer?.Dispose();
        }
    }
}

