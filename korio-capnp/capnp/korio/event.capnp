# Copyright 2026, Brian Olsen <brian@maven-group.org>
# Original protobuf schema: Copyright 2020, OpenTelemetry Authors
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
@0xfea8520c1e61d9a9;

using Common = import "/korio/common.capnp";


struct Event {
    # A log record according to OpenTelemetry Log Data Model:
    # https://github.com/open-telemetry/oteps/blob/main/text/logs/0097-log-data-model.md

    timeUnixNano @0 :Common.UnixTimeNano;
    # time_unix_nano is the time when the event occurred.
    # Value is UNIX Epoch time in nanoseconds since 00:00:00 UTC on 1 January 1970.
    # Value of 0 indicates unknown or missing timestamp.

    observedTimeUnixNano @1 :Common.UnixTimeNano;
    # Time when the event was observed by the collection system.
    # For events that originate in OpenTelemetry (e.g. using OpenTelemetry Logging SDK)
    # this timestamp is typically set at the generation time and is equal to Timestamp.
    # For events originating externally and collected by OpenTelemetry (e.g. using
    # Collector) this is the time when OpenTelemetry's code observed the event measured
    # by the clock of the OpenTelemetry code. This field MUST be set once the event is
    # observed by OpenTelemetry.
    #
    # For converting OpenTelemetry log data to formats that support only one timestamp or
    # when receiving OpenTelemetry log data by recipients that support only one timestamp
    # internally the following logic is recommended:
    #   - Use time_unix_nano if it is present, otherwise use observed_time_unix_nano.
    #
    # Value is UNIX Epoch time in nanoseconds since 00:00:00 UTC on 1 January 1970.
    # Value of 0 indicates unknown or missing timestamp.

    severityNumber @2 :Common.Severity;
    # Numerical value of the severity, normalized to values described in Log Data Model.
    # [Optional].

    severityText @3 :Text;
    # The severity text (also known as log level). The original string representation as
    # it is known at the source. [Optional].

    body @4 :Common.AnyValue;
    # A value containing the body of the log record. Can be for example a human-readable
    # string message (including multi-line) describing the event in a free form or it can
    # be a structured data composed of arrays and maps of other values. [Optional].

    attributes @5 :List(Common.KeyValue);
    # Additional attributes that describe the specific event occurrence. [Optional].
    # Attribute keys MUST be unique (it is not allowed to have more than one
    # attribute with the same key).
    # The behavior of software that receives duplicated keys can be unpredictable.

    droppedAttributesCount @6 :UInt32;
    # The number of dropped attributes. If the value is 0,
    # then no attributes were dropped.

    flags @7 :UInt32;
    # Flags, a bit field. 8 least significant bits are the trace flags as
    # defined in W3C Trace Context specification. 24 most significant bits are reserved
    # and must be set to 0. Readers must not assume that 24 most significant bits
    # will be zero and must correctly mask the bits when reading 8-bit trace flag (use
    # flags & LOG_RECORD_FLAGS_TRACE_FLAGS_MASK). [Optional].

    traceId @8 :Common.TraceId;
    # A unique identifier for a trace. All logs from the same trace share
    # the same `traceId`. The ID is a 16-byte array. An ID with all zeroes OR
    # of length other than 16 bytes is considered invalid (empty string in OTLP/JSON
    # is zero-length and thus is also invalid).
    #
    # This field is optional.
    #
    # The receivers SHOULD assume that the log record is not associated with a
    # trace if any of the following is true:
    #   - the field is not present,
    #   - the field contains an invalid value.

    spanId @9 :Common.SpanId;
    # A unique identifier for a span within a trace, assigned when the span
    # is created. The ID is an 8-byte array. An ID with all zeroes OR of length
    # other than 8 bytes is considered invalid (empty string in OTLP/JSON
    # is zero-length and thus is also invalid).
    #
    # This field is optional. If the sender specifies a valid span_id then it SHOULD also
    # specify a valid trace_id.
    #
    # The receivers SHOULD assume that the log record is not associated with a
    # span if any of the following is true:
    #   - the field is not present,
    #   - the field contains an invalid value.

    name @10 :Text;
    # A unique identifier of event category/type.
    # All events with the same event_name are expected to conform to the same
    # schema for both their attributes and their body.
    #
    # Recommended to be fully qualified and short (no longer than 256 characters).
    #
    # Presence of event_name on the log record identifies this record
    # as an event.
    # This field is semantically required to be set to non-empty string.
    # Empty value is equivalent to an unknown span target.
    #
    # This field is required.

    target @11 :Text;
    # A unique identifier of event category/type.
    # All events with the same event_name are expected to conform to the same
    # schema for both their attributes and their body.
    #
    # Recommended to be fully qualified and short (no longer than 256 characters).
    #
    # This field is semantically required to be set to non-empty string.
    # Empty value is equivalent to an unknown span target.
    #
    # This field is required.
}
