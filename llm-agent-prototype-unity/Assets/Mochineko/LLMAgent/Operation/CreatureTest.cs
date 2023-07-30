#nullable enable
using System;
using System.Collections.Generic;
using Cysharp.Net.Http;
using Cysharp.Threading.Tasks;
using Mochineko.LLMAgent.Creature;
using Mochineko.LLMAgent.Creature.Generated;
using UniRx;
using Unity.Logging;
using UnityEngine;
using Motion = Mochineko.LLMAgent.Creature.Generated.Motion;

namespace Mochineko.LLMAgent.Operation
{
    internal sealed class CreatureTest : MonoBehaviour
    {
        [SerializeField]
        private string message = string.Empty;

        [SerializeField]
        private Animator? animator = null;

        private CreatureClient? client;

        private const string Address = "https://127.0.0.1:50051";
        private static readonly YetAnotherHttpHandler httpHandler = new()
        {
            SkipCertificateVerification = true, // Does not use client certification
        };
        private static readonly int animationId = Animator.StringToHash("animation");

        private void Awake()
        {
            Logging.Initialize();
        }

        private void Start()
        {
            if (animator == null)
            {
                throw new NullReferenceException(nameof(animator));
            }

            client = new CreatureClient(Address, httpHandler);

            client
                .OnStateReceived
                .Subscribe(OnStateReceived)
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

        private void OnStateReceived(State state)
        {
            if (animator == null)
            {
                throw new NullReferenceException(nameof(animator));
            }

            Log.Info("[LLMAgent.Operation] Received state: {0}, {1}, {2}",
                state.Emotion, state.Motion, state.Cry);

            if (unicornMotionMap.TryGetValue(state.Motion, out var motionIndex))
            {
                animator.SetInteger(animationId, motionIndex);
            }
            else
            {
                animator.SetInteger(animationId, value: 2);
            }
        }

        private static readonly IReadOnlyDictionary<Motion, int> unicornMotionMap = new Dictionary<Motion, int>
        {
            [Motion.Neutral] = 1,
            [Motion.Happy] = 3,
            [Motion.No] = 4,
            [Motion.Jump] = 11,
            [Motion.Die] = 10,
            [Motion.Run] = 5,
            [Motion.Walk] = 6,
            [Motion.Flying] = 9,
            [Motion.Attack] = 8,
            [Motion.Eating] = 7,
        };
    }
}
