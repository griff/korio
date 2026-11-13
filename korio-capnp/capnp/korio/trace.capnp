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
@0xd9d3d2786577997b;

using Common = import "/korio/common.capnp";

struct Span {
    # A Span represents a single operation performed by a single component of the system.
    #
    # The next available field id is 17.

    traceId @0 :Common.TraceId;
    # A unique identifier for a trace. All spans from the same trace share
    # the same `trace_id`. The ID is a 16-byte array. An ID with all zeroes OR
    # of length other than 16 bytes is considered invalid (empty string in OTLP/JSON
    # is zero-length and thus is also invalid).
    #
    # This field is required.

    spanId @1 :Common.SpanId;
    # A unique identifier for a span within a trace, assigned when the span
    # is created. The ID is an 8-byte array. An ID with all zeroes OR of length
    # other than 8 bytes is considered invalid (empty string in OTLP/JSON
    # is zero-length and thus is also invalid).
    #
    # This field is required.

    traceState @2 :Text;
    # trace_state conveys information about request position in multiple distributed tracing graphs.
    # It is a trace_state in w3c-trace-context format: https://www.w3.org/TR/trace-context/#tracestate-header
    # See also https://github.com/w3c/distributed-tracing for more details about this field.

    parentSpanId @3 :Common.SpanId;
    # The `span_id` of this span's parent span. If this is a root span, then this
    # field must be empty. The ID is an 8-byte array.

    flags @4 :UInt32;
    # Flags, a bit field.
    #
    # Bits 0-7 (8 least significant bits) are the trace flags as defined in W3C Trace
    # Context specification. To read the 8-bit W3C trace flag, use
    # `flags & SPAN_FLAGS_TRACE_FLAGS_MASK`.
    #
    # See https://www.w3.org/TR/trace-context-2/#trace-flags for the flag definitions.
    #
    # Bits 8 and 9 represent the 3 states of whether a span's parent
    # is remote. The states are (unknown, is not remote, is remote).
    # To read whether the value is known, use `(flags & SPAN_FLAGS_CONTEXT_HAS_IS_REMOTE_MASK) != 0`.
    # To read whether the span is remote, use `(flags & SPAN_FLAGS_CONTEXT_IS_REMOTE_MASK) != 0`.
    #
    # When creating span messages, if the message is logically forwarded from another source
    # with an equivalent flags fields (i.e., usually another OTLP span message), the field SHOULD
    # be copied as-is. If creating from a source that does not have an equivalent flags field
    # (such as a runtime representation of an OpenTelemetry span), the high 22 bits MUST
    # be set to zero.
    # Readers MUST NOT assume that bits 10-31 (22 most significant bits) will be zero.
    #
    # [Optional].

    name @5 :Text;
    # A description of the span's operation.
    #
    # For example, the name can be a qualified method name or a file name
    # and a line number where the operation is called. A best practice is to use
    # the same display name at the same call point in an application.
    # This makes it easier to correlate spans in different traces.
    #
    # This field is semantically required to be set to non-empty string.
    # Empty value is equivalent to an unknown span name.
    #
    # This field is required.

    target @6 :Text;
    # A description of the span's operation.
    #
    # For example, the name can be a qualified method name or a file name
    # and a line number where the operation is called. A best practice is to use
    # the same display name at the same call point in an application.
    # This makes it easier to correlate spans in different traces.
    #
    # This field is semantically required to be set to non-empty string.
    # Empty value is equivalent to an unknown span target.
    #
    # This field is required.

    severityNumber @7 :Common.Severity;
    # Numerical value of the severity, normalized to values described in Log Data Model.
    # [Optional].

    severityText @8 :Text;
    # The severity text (also known as log level). The original string representation as
    # it is known at the source. [Optional].


    startTimeUnixNano @9 :Common.UnixTimeNano;
    # The start time of the span. On the client side, this is the time
    # kept by the local machine where the span execution starts. On the server side, this
    # is the time when the server's application handler starts running.
    # Value is UNIX Epoch time in nanoseconds since 00:00:00 UTC on 1 January 1970.
    #
    # This field is semantically required and it is expected that end_time >= start_time.

    endTimeUnixNano @10 :Common.UnixTimeNano;
    # The end time of the span. On the client side, this is the time
    # kept by the local machine where the span execution ends. On the server side, this
    # is the time when the server application handler stops running.
    # Value is UNIX Epoch time in nanoseconds since 00:00:00 UTC on 1 January 1970.
    #
    # This field is optional with a value of 0 meaning not present. It is expected that if
    # end_time is present then end_time >= start_time.


    attributes @11 :List(Common.KeyValue);
    # A collection of key/value pairs. Note, global attributes
    # like server name can be set using the resource API. Examples of attributes:
    #
    #     "/http/user_agent": "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_14_2) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/71.0.3578.98 Safari/537.36"
    #     "/http/server_latency": 300
    #     "example.com/myattribute": true
    #     "example.com/score": 10.239
    #
    # Attribute keys MUST be unique (it is not allowed to have more than one
    # attribute with the same key).
    # The behavior of software that receives duplicated keys can be unpredictable.
}

struct SpanFollowsFrom {
    # A pointer from the current span to another span in the same trace or in a
    # different trace. For example, this can be used in batching operations,
    # where a single batch handler processes multiple requests from different
    # traces or when the handler receives a request from a different project.

    id :group {
        traceId @0 :Common.TraceId;
        # A unique identifier of a trace that this linked span is part of. The ID is a
        # 16-byte array.

        spanId @1 :Common.SpanId;
        # A unique identifier for the linked span. The ID is an 8-byte array.
    }

    follows :group {
        traceId @2 :Common.TraceId;
        # A unique identifier of a trace that this linked span is part of. The ID is a
        # 16-byte array.

        spanId @3 :Common.SpanId;
        # A unique identifier for the linked span. The ID is an 8-byte array.
    }

    observedTimeUnixNano @4 :Common.UnixTimeNano;
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

    attributes @5 :List(Common.KeyValue);
    # A collection of attribute key/value pairs on the link.
    # Attribute keys MUST be unique (it is not allowed to have more than one
    # attribute with the same key).
    # The behavior of software that receives duplicated keys can be unpredictable.

    droppedAttributesCount @6 :UInt32;
    # The number of dropped attributes. If the value is 0,
    # then no attributes were dropped.

    traceState @7 :Text;
    # The trace_state associated with the link.

    flags @8 :UInt32;
    # Flags, a bit field.
    #
    # Bits 0-7 (8 least significant bits) are the trace flags as defined in W3C Trace
    # Context specification. To read the 8-bit W3C trace flag, use
    # `flags & SPAN_FLAGS_TRACE_FLAGS_MASK`.
    #
    # See https://www.w3.org/TR/trace-context-2/#trace-flags for the flag definitions.
    #
    # Bits 8 and 9 represent the 3 states of whether the link is remote.
    # The states are (unknown, is not remote, is remote).
    # To read whether the value is known, use `(flags & SPAN_FLAGS_CONTEXT_HAS_IS_REMOTE_MASK) != 0`.
    # To read whether the link is remote, use `(flags & SPAN_FLAGS_CONTEXT_IS_REMOTE_MASK) != 0`.
    #
    # Readers MUST NOT assume that bits 10-31 (22 most significant bits) will be zero.
    # When creating new spans, bits 10-31 (most-significant 22-bits) MUST be zero.
    #
    # [Optional].
}
