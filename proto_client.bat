@echo off
cd /d %~dp0
set BUILD_PATH=llm-agent-prototype-unity/Assets/Mochineko/LLMAgent/Creature/Generated
set PLUGIN_PATH=../../Apps/Protoc/grpc_csharp_plugin.exe
set PROTO_PATH=llm-agent-prototype-server/proto/creature.proto
protoc --csharp_out %BUILD_PATH% --grpc_out %BUILD_PATH% --plugin=protoc-gen-grpc=%PLUGIN_PATH% %PROTO_PATH%