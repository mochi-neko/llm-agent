#nullable enable
using Unity.Logging;
using Unity.Logging.Sinks;

namespace Mochineko.LLMAgent.Operation
{
    public static class Logging
    {
        public static void Initialize()
        {
            Log.Logger =
#if UNITY_EDITOR
                EditorConfiguration().CreateLogger();
#elif DEBUG
                DevelopmentConfiguration().CreateLogger();
#else
                ReleaseConfiguration().CreateLogger();
#endif

            Log.Info("[LLMAgent.Operation] Setup logging.");
        }

        private static LoggerConfig EditorConfiguration()
            => new LoggerConfig()
                .SyncMode.FatalIsSync()
                //.RedirectUnityLogs(log:true)
                .WriteTo.UnityEditorConsole(
                    minLevel:LogLevel.Info,
                    captureStackTrace:true)
                .WriteTo.File(
                    absFileName:$"{UnityEngine.Application.dataPath}/../Logs/logging/llm-agent-prototype-unity_editor_{System.DateTime.Now:yyyy-MM-dd_HH-mm-ss}.log",
                    minLevel:LogLevel.Debug,
                    captureStackTrace:true,
                    outputTemplate:"{Timestamp} [{Level}] {Message}{NewLine}{Stacktrace}");

        private static LoggerConfig DevelopmentConfiguration()
            => new LoggerConfig()
                .SyncMode.FatalIsSync()
                //.RedirectUnityLogs(log:true)
                .WriteTo.File(
                    absFileName:$"{UnityEngine.Application.dataPath}/../Logs/logging_dev/llm-agent-prototype-unity_dev_{System.DateTime.Now:yyyy-MM-dd_HH-mm-ss}.log",
                    minLevel:LogLevel.Debug,
                    captureStackTrace:true,
                    outputTemplate:"{Timestamp} [{Level}] {Message}{NewLine}{Stacktrace}");

        private static LoggerConfig ReleaseConfiguration()
            => new LoggerConfig()
                .SyncMode.FatalIsSync()
                //.RedirectUnityLogs(log:true)
                .WriteTo.File(
                    absFileName:$"{UnityEngine.Application.persistentDataPath}/Logs/logging/llm-agent-prototype-unity_release_{System.DateTime.Now:yyyy-MM-dd_HH-mm-ss}.log",
                    minLevel:LogLevel.Info,
                    captureStackTrace:false,
                    outputTemplate:"{Timestamp} [{Level}] {Message}");
    }
}
