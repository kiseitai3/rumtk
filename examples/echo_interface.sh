#!/bin/bash
#
#     rumtk attempts to implement HL7 and medical protocols for interoperability in medicine.
#     This toolkit aims to be reliable, simple, performant, and standards compliant.
#     Copyright (C) 2025  Luis M. Santos, M.D.
#
#     This program is free software: you can redistribute it and/or modify
#     it under the terms of the GNU General Public License as published by
#     the Free Software Foundation, either version 3 of the License, or
#     (at your option) any later version.
#
#     This program is distributed in the hope that it will be useful,
#     but WITHOUT ANY WARRANTY; without even the implied warranty of
#     MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
#     GNU General Public License for more details.
#
#     You should have received a copy of the GNU General Public License
#     along with this program.  If not, see <https://www.gnu.org/licenses/>.
#

mkdir demo

echo "Setting up Interface Chain"
#./target/debug/rumtk-v2-interface --port 55555 --local > demo/out.log &
sleep 1
./target/debug/rumtk-v2-interface --port 55556 --local | ./target/debug/rumtk-v2-interface --outbound --port 55555 --local &
sleep 1

echo "Pushing Message through PIPEs!"
cat examples/sample_hl7.hl7 | ./target/debug/rumtk-v2-interface --outbound --local --port 55556

sleep 20

echo "Output"
cat demo/out.log

echo "Clean up"
pkill -i -e -f rumtk-v2-interface
#rm -r demo
