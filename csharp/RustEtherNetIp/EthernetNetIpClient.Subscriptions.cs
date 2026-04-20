using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading;
using System.Threading.Tasks;

namespace RustEtherNetIp
{
    public partial class EtherNetIpClient
    {
        /// <summary>
        /// Subscribes to a tag for real-time updates.
        /// </summary>
        public TagSubscription SubscribeToTag(string tagName, SubscriptionOptions? options = null)
        {
            if (_isDisposed)
                throw new ObjectDisposedException(nameof(EtherNetIpClient));

            options ??= new SubscriptionOptions();
            var initialValue = ReadTagValueAsync(tagName).GetAwaiter().GetResult();

            lock (_subscriptionLock)
            {
                if (_subscriptions.ContainsKey(tagName))
                    return _subscriptions[tagName];

                var subscription = new TagSubscription(tagName);
                subscription.UpdateValue(initialValue);
                _subscriptions[tagName] = subscription;

                var cts = new CancellationTokenSource();
                _subscriptionTokens[tagName] = cts;

                Task.Run(async () =>
                {
                    int reconnectAttempts = 0;
                    while (!cts.Token.IsCancellationRequested)
                    {
                        try
                        {
                            var value = await ReadTagValueAsync(tagName);
                            subscription.UpdateValue(value);

                            if (reconnectAttempts > 0)
                                reconnectAttempts = 0;

                            await Task.Delay(options.PollIntervalMs, cts.Token);
                        }
                        catch (Exception)
                        {
                            if (!options.AutoReconnect || reconnectAttempts >= options.MaxReconnectAttempts)
                                break;

                            reconnectAttempts++;
                            await Task.Delay(options.ReconnectDelayMs, cts.Token);
                        }
                    }
                }, cts.Token);

                return subscription;
            }
        }

        public void UnsubscribeFromTag(string tagName)
        {
            if (_isDisposed)
                throw new ObjectDisposedException(nameof(EtherNetIpClient));

            lock (_subscriptionLock)
            {
                if (_subscriptionTokens.TryGetValue(tagName, out var cts))
                {
                    cts.Cancel();
                    cts.Dispose();
                    _subscriptionTokens.Remove(tagName);
                }

                _subscriptions.Remove(tagName);
            }
        }

        public void UnsubscribeFromAllTags()
        {
            if (_isDisposed)
                throw new ObjectDisposedException(nameof(EtherNetIpClient));

            lock (_subscriptionLock)
            {
                foreach (var cts in _subscriptionTokens.Values)
                {
                    cts.Cancel();
                    cts.Dispose();
                }

                _subscriptionTokens.Clear();
                _subscriptions.Clear();
            }
        }

        private Task<object> ReadTagValueAsync(string tagName)
        {
            return Task.Run(() => (object)ConvertPlcValueToObject(ReadTag(tagName)));
        }

        public void UpsertTagGroup(string groupName, string[] tagNames, int updateRateMs = 500)
        {
            if (_isDisposed)
                throw new ObjectDisposedException(nameof(EtherNetIpClient));
            if (string.IsNullOrWhiteSpace(groupName))
                throw new ArgumentException("Group name cannot be empty", nameof(groupName));
            if (tagNames == null || tagNames.Length == 0)
                throw new ArgumentException("Tag group must contain at least one tag", nameof(tagNames));
            if (updateRateMs <= 0)
                throw new ArgumentException("Update rate must be greater than zero", nameof(updateRateMs));

            lock (_tagGroupLock)
            {
                if (_tagGroups.TryGetValue(groupName, out var existing))
                {
                    existing.Group?.Stop();
                    existing.Group?.Dispose();
                }

                _tagGroups[groupName] = new TagGroupRegistration
                {
                    GroupName = groupName,
                    TagNames = tagNames.ToArray(),
                    UpdateRateMs = updateRateMs
                };
            }
        }

        public bool RemoveTagGroup(string groupName)
        {
            if (_isDisposed)
                throw new ObjectDisposedException(nameof(EtherNetIpClient));

            lock (_tagGroupLock)
            {
                if (!_tagGroups.TryGetValue(groupName, out var registration))
                    return false;

                registration.Group?.Stop();
                registration.Group?.Dispose();
                _tagGroups.Remove(groupName);
                return true;
            }
        }

        public List<TagGroupConfigInfo> ListTagGroups()
        {
            if (_isDisposed)
                throw new ObjectDisposedException(nameof(EtherNetIpClient));

            lock (_tagGroupLock)
            {
                return _tagGroups.Values
                    .Select(g => new TagGroupConfigInfo
                    {
                        GroupName = g.GroupName,
                        TagNames = g.TagNames.ToArray(),
                        UpdateRateMs = g.UpdateRateMs,
                        IsActive = g.Group?.IsActive == true
                    })
                    .ToList();
            }
        }

        public TagGroupSnapshot ReadTagGroupOnce(string groupName)
        {
            if (_isDisposed)
                throw new ObjectDisposedException(nameof(EtherNetIpClient));

            TagGroupRegistration registration;
            lock (_tagGroupLock)
            {
                if (!_tagGroups.TryGetValue(groupName, out registration!))
                    throw new InvalidOperationException($"Tag group '{groupName}' is not registered");
            }

            var batch = ReadTagsBatch(registration.TagNames);
            var snapshot = new TagGroupSnapshot
            {
                GroupName = groupName,
                SampledAt = DateTime.UtcNow
            };

            foreach (var kvp in batch)
            {
                if (kvp.Value.Success)
                    snapshot.Values[kvp.Key] = kvp.Value.Value;
                else
                    snapshot.Errors[kvp.Key] = kvp.Value.ErrorMessage ?? "Unknown error";
            }

            return snapshot;
        }

        public TagGroup SubscribeToTagGroup(string groupName)
        {
            if (_isDisposed)
                throw new ObjectDisposedException(nameof(EtherNetIpClient));

            lock (_tagGroupLock)
            {
                if (!_tagGroups.TryGetValue(groupName, out var registration))
                    throw new InvalidOperationException($"Tag group '{groupName}' is not registered");

                if (registration.Group != null)
                    return registration.Group;

                var group = new TagGroup(this)
                {
                    TagNames = registration.TagNames.ToArray(),
                    UpdateRateMs = registration.UpdateRateMs
                };
                group.Start();
                registration.Group = group;
                return group;
            }
        }
    }
}
