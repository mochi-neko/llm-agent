#nullable enable
using Cysharp.Threading.Tasks;
using GRPC.NET;
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
        private static readonly GRPCBestHttpHandler httpHandler = new();

        private void Start()
        {
            client = new CreatureClient(address, httpHandler);

            client
                .OnStateReceived
                .Subscribe(state => Log.Debug(
                    "[LLMAgent.Creature] Received state: {0}, {1}, {2}",
                    state.Emotion, state.Motion, state.Cry))
                .AddTo(this);
        }

        private void OnDestroy()
        {
            client?.Dispose();
        }

        [ContextMenu(nameof(Send))]
        public void Send()
        {
            client?.Send(new Talking()
                {
                    Message = message
                })
                .Forget();
        }
    }
}
