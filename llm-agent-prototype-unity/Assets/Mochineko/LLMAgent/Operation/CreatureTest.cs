#nullable enable
using Cysharp.Net.Http;
using Cysharp.Threading.Tasks;
using Mochineko.LLMAgent.Creature;
using Mochineko.LLMAgent.Creature.Generated;
using UniRx;
using Unity.Logging;
using UnityEngine;

namespace Mochineko.LLMAgent.Operation
{
    internal sealed class CreatureTest : MonoBehaviour
    {
        [SerializeField]
        private string address = "https://127.0.0.1:8000";

        [SerializeField]
        private string message = string.Empty;

        private CreatureClient? client;
        private static readonly YetAnotherHttpHandler httpHandler = new();

        private void Awake()
        {
            Logging.Initialize();
        }

        private void Start()
        {
            client = new CreatureClient(address, httpHandler);

            client
                .OnStateReceived
                .Subscribe(state => Log.Info(
                    "[LLMAgent.Operation] Received state: {0}, {1}, {2}",
                    state.Emotion, state.Motion, state.Cry))
                .AddTo(this);
        }

        private void OnDestroy()
        {
            client?.Dispose();
            Log.FlushAll();
        }

        [ContextMenu(nameof(Send))]
        public void Send()
        {
            // Log.Info("[LLMAgent.Operation] Send message: {0}", message);
            Debug.LogFormat("[LLMAgent.Operation] Send message: {0}", message);

            client?.Send(new Talking
                    {
                        Message = message
                    },
                    this.GetCancellationTokenOnDestroy())
                .Forget();
        }
    }
}
