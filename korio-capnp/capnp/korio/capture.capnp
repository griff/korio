# Copyright 2026, Brian Olsen <brian@maven-group.org>
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
@0xe271aeb30089a232;

using Common = import "common.capnp";
using Event = import "event.capnp";
using Trace = import "trace.capnp";

struct SpanRecord {
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

    observedTimeUnixNano @2 :Common.UnixTimeNano;
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

    attributes @3 :List(Common.KeyValue);
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

    droppedAttributesCount @4 :UInt32;
    # The number of attributes that were discarded. Attributes
    # can be discarded because their keys are too long or because there are too many
    # attributes. If this value is 0, then no attributes were dropped.
}

struct TelemetryEvent {
    union {
        spanStart @0 :Trace.Span;
        spanRecord @1 :SpanRecord;
        spanRecordFollowsFrom @2 :Trace.SpanFollowsFrom;
        spanClose :group {
            traceId @3 :Common.TraceId;
            # A unique identifier for a trace. All spans from the same trace share
            # the same `trace_id`. The ID is a 16-byte array. An ID with all zeroes OR
            # of length other than 16 bytes is considered invalid (empty string in OTLP/JSON
            # is zero-length and thus is also invalid).
            #
            # This field is required.

            spanId @4 :Common.SpanId;
            # A unique identifier for a span within a trace, assigned when the span
            # is created. The ID is an 8-byte array. An ID with all zeroes OR of length
            # other than 8 bytes is considered invalid (empty string in OTLP/JSON
            # is zero-length and thus is also invalid).
            #
            # This field is required.

            endTimeUnixNano @5 :Common.UnixTimeNano;
            # The end time of the span. On the client side, this is the time
            # kept by the local machine where the span execution ends. On the server side, this
            # is the time when the server application handler stops running.
            # Value is UNIX Epoch time in nanoseconds since 00:00:00 UTC on 1 January 1970.
            #
            # This field is semantically required and it is expected that end_time >= start_time.
        }
        event @6 :Event.Event;
    }
}

interface TelemetryStream {
    events @0 (events :List(TelemetryEvent)) -> stream;
    completed @1 ();
}

struct LogsFilter {
}

struct TracesFilter {
}

struct TelemetryConfig {
    logsFilter @0 :LogsFilter;
    tracesFilter @1 :TracesFilter;

    traceId @2 :Common.TraceId;
    # A unique identifier for the upstream trace that this capture belongs to.
    #
    # This field is required.

    spanId @3 :Common.SpanId;
    # A unique identifier for the upstream span that this capture belongs to.
    #
    # This field is required.
}

struct TelemetryCapture {
    receiver @0 :TelemetryStream;
    config @1 :TelemetryConfig;
}

interface Capture {
    capture @0 (capture :TelemetryCapture) -> (wrapped :Capability);
}
