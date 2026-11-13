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
@0xf545c653755eb519;

using Common = import "/korio/common.capnp";

struct UnixTimeNanoRange {
    start @0 :Common.UnixTimeNano;
    end @1 :Common.UnixTimeNano;
}

struct TimedStreamHeader {
    timeRange @0 :UnixTimeNanoRange;
}

struct TimedEvent(Event) {
    timestamp @0 :Common.UnixTimeNano;
    event @1 :Event;
}

struct TimedRange(Event) {
    range @0 :UnixTimeNanoRange;
    event @1 :Event;
}
