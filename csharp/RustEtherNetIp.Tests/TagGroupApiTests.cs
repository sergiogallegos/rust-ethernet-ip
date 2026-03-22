using System;
using Xunit;
using RustEtherNetIp;

namespace RustEtherNetIp.Tests
{
    public class TagGroupApiTests
    {
        [Fact]
        public void UpsertTagGroup_WithInvalidArguments_Throws()
        {
            using var client = new EtherNetIpClient();

            Assert.Throws<ArgumentException>(() => client.UpsertTagGroup("", new[] { "Tag1" }));
            Assert.Throws<ArgumentException>(() => client.UpsertTagGroup("GroupA", Array.Empty<string>()));
            Assert.Throws<ArgumentException>(() => client.UpsertTagGroup("GroupA", new[] { "Tag1" }, 0));
        }

        [Fact]
        public void UpsertAndListTagGroups_ReturnsConfiguredGroup()
        {
            using var client = new EtherNetIpClient();

            client.UpsertTagGroup("Core", new[] { "Tag1", "Tag2" }, 250);
            var groups = client.ListTagGroups();

            var group = Assert.Single(groups);
            Assert.Equal("Core", group.GroupName);
            Assert.Equal(250, group.UpdateRateMs);
            Assert.Equal(new[] { "Tag1", "Tag2" }, group.TagNames);
            Assert.False(group.IsActive);
        }

        [Fact]
        public void SubscribeToTagGroup_ReturnsSameInstanceForExistingSubscription()
        {
            using var client = new EtherNetIpClient();
            client.UpsertTagGroup("LineScan", new[] { "Tag1" }, 1000);

            var first = client.SubscribeToTagGroup("LineScan");
            var second = client.SubscribeToTagGroup("LineScan");

            Assert.Same(first, second);
            Assert.True(first.IsActive);
        }

        [Fact]
        public void RemoveTagGroup_StopsAndRemovesActiveGroup()
        {
            using var client = new EtherNetIpClient();
            client.UpsertTagGroup("LineScan", new[] { "Tag1" }, 1000);
            var group = client.SubscribeToTagGroup("LineScan");
            Assert.True(group.IsActive);

            var removed = client.RemoveTagGroup("LineScan");
            var groups = client.ListTagGroups();

            Assert.True(removed);
            Assert.False(group.IsActive);
            Assert.Empty(groups);
        }

        [Fact]
        public void RemoveTagGroup_WhenMissing_ReturnsFalse()
        {
            using var client = new EtherNetIpClient();
            Assert.False(client.RemoveTagGroup("Missing"));
        }

        [Fact]
        public void SubscribeToTagGroup_WhenUnregistered_Throws()
        {
            using var client = new EtherNetIpClient();
            var ex = Assert.Throws<InvalidOperationException>(() => client.SubscribeToTagGroup("Missing"));
            Assert.Contains("is not registered", ex.Message, StringComparison.OrdinalIgnoreCase);
        }

        [Fact]
        public void ReadTagGroupOnce_WhenUnregistered_Throws()
        {
            using var client = new EtherNetIpClient();
            var ex = Assert.Throws<InvalidOperationException>(() => client.ReadTagGroupOnce("Missing"));
            Assert.Contains("is not registered", ex.Message, StringComparison.OrdinalIgnoreCase);
        }

        [Fact]
        public void ReadTagGroupOnce_WhenNotConnected_ReturnsSnapshotWithErrors()
        {
            using var client = new EtherNetIpClient();
            client.UpsertTagGroup("Core", new[] { "Tag1", "Tag2" }, 250);

            var snapshot = client.ReadTagGroupOnce("Core");

            Assert.Equal("Core", snapshot.GroupName);
            Assert.Empty(snapshot.Values);
            Assert.Equal(2, snapshot.Errors.Count);
            Assert.True(snapshot.HasErrors);
        }
    }
}
