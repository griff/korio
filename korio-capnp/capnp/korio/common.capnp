# Copyright 2026, Brian Olsen <brian@maven-group.org>
# Original protobuf schema: Copyright 2019, OpenTelemetry Authors
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.
@0xb40eafabe6638c3c;

struct AnyValue {
    # Represents any type of attribute value. AnyValue may contain a
    # primitive value such as a string or integer or it may contain an arbitrary nested
    # object containing arrays, key-value lists and primitives.

    union {
        empty @0 :Void;
        stringValue @1 :Text;
        boolValue @2 :Bool;
        int8Value @3 :Int8;
        int16Value @4 :Int16;
        int32Value @5 :Int32;
        int64Value @6 :Int64;
        int128Value @7 :Int128;

        uint8Value @8 :UInt8;
        uint16Value @9 :UInt16;
        uint32Value @10 :UInt32;
        uint64Value @11 :UInt64;
        uint128Value @12 :UInt128;

        float32Value @13 :Float32;
        float64Value @14 :Float64;

        arrayValue @15 :List(AnyValue);
        # Array of values. The array may be empty (contain 0 elements).

        kvlistValue @16 :List(KeyValue);
        # A collection of key/value pairs of key-value pairs. The list may be empty (may
        # contain 0 elements).
        #
        # The keys MUST be unique (it is not allowed to have more than one
        # value with the same key).
        # The behavior of software that receives duplicated keys can be unpredictable.

        bytesValue @17 :Data;
    }
}

struct KeyValue {
    # Represents a key-value pair that is used to store Span attributes, Link
    # attributes, etc.

    key @0 :Text;
    # The key name of the pair.

    value @1 :AnyValue;
    # The value of the pair.
}

enum Severity {
    unspecified @0;
    trace  @1;
    trace2 @2;
    trace3 @3;
    trace4 @4;
    debug  @5;
    debug2 @6;
    debug3 @7;
    debug4 @8;
    info   @9;
    info2  @10;
    info3  @11;
    info4  @12;
    warn   @13;
    warn2  @14;
    warn3  @15;
    warn4  @16;
    error  @17;
    error2 @18;
    error3 @19;
    error4 @20;
    fatal  @21;
    fatal2 @22;
    fatal3 @23;
    fatal4 @24;
}

struct UInt128 {
    upper64 @0 :UInt64;
    lower64 @1 :UInt64;
}

struct Int128 {
    upper64 @0 :UInt64;
    lower64 @1 :UInt64;
}

struct TraceId {
    # A unique identifier for a trace. All logs from the same trace share
    # the same `traceId`. The ID is a 16-byte array. An ID with all zeroes
    # is considered invalid.

    upper64 @0 :UInt64;
    lower64 @1 :UInt64;
}

using SpanId = UInt64;
# A unique identifier for a span within a trace, assigned when the span
# is created. The ID is an 8-byte array. An ID with all zeroes OR of length
# other than 8 bytes is considered invalid (empty string in OTLP/JSON
# is zero-length and thus is also invalid).

using UnixTimeNano = UInt64;
using RelativeTimeNano = UInt64;
using DurationNano = UInt64;
