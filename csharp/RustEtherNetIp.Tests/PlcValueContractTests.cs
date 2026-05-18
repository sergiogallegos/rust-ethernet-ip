using System;
using System.Collections.Generic;
using Xunit;

namespace RustEtherNetIp.Tests
{
    public class PlcValueContractTests
    {
        [Theory]
        [InlineData("{\"Bool\":true}", PlcValueType.Bool, true)]
        [InlineData("{\"Sint\":-12}", PlcValueType.Sint, (sbyte)-12)]
        [InlineData("{\"Int\":-1234}", PlcValueType.Int, (short)-1234)]
        [InlineData("{\"Dint\":123456}", PlcValueType.Dint, 123456)]
        [InlineData("{\"Lint\":1234567890123}", PlcValueType.Lint, 1234567890123L)]
        [InlineData("{\"Usint\":250}", PlcValueType.Usint, (byte)250)]
        [InlineData("{\"Uint\":65000}", PlcValueType.Uint, (ushort)65000)]
        [InlineData("{\"Udint\":4000000000}", PlcValueType.Udint, 4000000000U)]
        [InlineData("{\"Ulint\":9000000000000}", PlcValueType.Ulint, 9000000000000UL)]
        [InlineData("{\"Real\":3.5}", PlcValueType.Real, 3.5f)]
        [InlineData("{\"Lreal\":6.25}", PlcValueType.Lreal, 6.25d)]
        [InlineData("{\"String\":\"hello\"}", PlcValueType.String, "hello")]
        public void FromJson_ParsesRustEnumShapes(string json, PlcValueType expectedType, object expectedValue)
        {
            var value = PlcValue.FromJson(json);

            Assert.Equal(expectedType, value.Type);
            Assert.Equal(expectedValue, value.Value);
        }

        [Fact]
        public void FromJson_ParsesRawUdtDataShape()
        {
            var value = PlcValue.FromJson("{\"Udt\":{\"symbol_id\":77,\"data\":[0,1,127,255]}}");

            Assert.Equal(PlcValueType.Udt, value.Type);
            Assert.True(value.IsUdtDataFormat);
            Assert.Null(value.UdtMembers);
            Assert.NotNull(value.UdtData);
            Assert.Equal(77, value.UdtData!.SymbolId);
            Assert.Equal(new byte[] { 0, 1, 127, 255 }, value.UdtData.Data);
        }

        [Fact]
        public void FromJson_ParsesNestedUdtDictionary()
        {
            var value = PlcValue.FromJson("""
                {
                  "Udt": {
                    "Running": {"Bool": true},
                    "Count": {"Dint": 42},
                    "Nested": {
                      "Status": {"String": "OK"}
                    }
                  }
                }
                """);

            Assert.True(value.IsUdt);
            Assert.NotNull(value.UdtMembers);
            Assert.True(value.GetNestedValue("Running")!.As<bool>());
            Assert.Equal(42, value.GetNestedValue("Count")!.As<int>());
            Assert.Equal("OK", value.GetNestedValue("Nested.Status")!.As<string>());
            Assert.Null(value.GetNestedValue("Nested.Missing"));
        }

        [Fact]
        public void FromJson_FallbackSimpleValues_AreTypedPredictably()
        {
            Assert.Equal(PlcValueType.Bool, PlcValue.FromJson("true").Type);
            Assert.Equal(PlcValueType.Dint, PlcValue.FromJson("123").Type);
            Assert.Equal(PlcValueType.Real, PlcValue.FromJson("3.25").Type);
            Assert.Equal(PlcValueType.String, PlcValue.FromJson("\"abc\"").Type);
        }

        [Fact]
        public void FromJson_InvalidPayloads_ThrowArgumentException()
        {
            Assert.Throws<ArgumentException>(() => PlcValue.FromJson("not-json"));
            Assert.Throws<ArgumentException>(() => PlcValue.FromJson("[1,2,3]"));
            Assert.Throws<ArgumentException>(() => PlcValue.FromJson("{\"Bool\":\"not-bool\"}"));
        }

        [Fact]
        public void AsOrDefault_ReturnsDefaultWhenRequestedTypeDoesNotMatch()
        {
            var value = PlcValue.Dint(42);

            Assert.Equal(42, value.AsOrDefault<int>());
            Assert.Equal("fallback", value.AsOrDefault("fallback"));
        }

        [Fact]
        public void UdtMembers_ReturnLegacyDictionaryForLegacyUdt()
        {
            var member = PlcValue.Dint(7);
            var value = PlcValue.Udt(new Dictionary<string, PlcValue> { ["Count"] = member });

            Assert.True(value.IsUdt);
            Assert.False(value.IsUdtDataFormat);
            Assert.Same(member, value.UdtMembers!["Count"]);
            Assert.Null(value.UdtData);
        }
    }
}
