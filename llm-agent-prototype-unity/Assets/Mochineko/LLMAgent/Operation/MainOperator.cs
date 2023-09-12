#nullable enable
using System;
using System.Collections.Generic;
using Cysharp.Net.Http;
using Cysharp.Threading.Tasks;
using Mochineko.LLMAgent.Creature;
using Mochineko.LLMAgent.Creature.Generated;
using SatorImaging.AppWindowUtility;
using TMPro;
using UniRx;
using Unity.Logging;
using UnityEngine;
using UnityEngine.UI;
using Motion = Mochineko.LLMAgent.Creature.Generated.Motion;

namespace Mochineko.LLMAgent.Operation
{
    internal sealed class MainOperator : MonoBehaviour
    {
        [SerializeField]
        private string author = "Mochineko";

        [SerializeField]
        private Button? inputToggle = null;

        [SerializeField]
        private GameObject? inputParent = null;

        [SerializeField]
        private TMP_InputField? messageInput = null;

        [SerializeField]
        private Button? sendMessageButton = null;

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

            AppWindowUtility.platform = new Windows();
            AppWindowUtility.FullScreen = false;
            AppWindowUtility.Transparent = true;
            AppWindowUtility.AlwaysOnTop = true;
            AppWindowUtility.FrameVisibility = false;
            AppWindowUtility.ClickThrough = false;
        }

        private void Start()
        {
            if (inputToggle == null)
            {
                throw new NullReferenceException(nameof(inputToggle));
            }

            if (inputParent == null)
            {
                throw new NullReferenceException(nameof(inputParent));
            }

            if (messageInput == null)
            {
                throw new NullReferenceException(nameof(messageInput));
            }

            if (sendMessageButton == null)
            {
                throw new NullReferenceException(nameof(sendMessageButton));
            }

            if (animator == null)
            {
                throw new NullReferenceException(nameof(animator));
            }

            client = new CreatureClient(Address, httpHandler);

            client
                .OnStateReceived
                .Subscribe(OnStateReceived)
                .AddTo(this);

            inputToggle
                .OnClickAsObservable()
                .Subscribe(_ =>
                {
                    Log.Info("[LLMAgent.Operation] On click input toggle button.");

                    Log.Info("[LLMAgent.Operation] Toggle into {0}", !inputParent.activeSelf ? "On" : "Off");
                    inputParent.SetActive(!inputParent.activeSelf);
                })
                .AddTo(this);

            sendMessageButton
                .OnClickAsObservable()
                .Subscribe(_ =>
                {
                    Log.Info("[LLMAgent.Operation] On click send message button.");
                    SendMessage();
                })
                .AddTo(this);
        }

        private void OnDestroy()
        {
            client?.Dispose();
            Log.FlushAll();
        }

        private void SendMessage()
        {
            if (messageInput == null)
            {
                throw new NullReferenceException(nameof(messageInput));
            }

            // Log.Info("[LLMAgent.Operation] Send message: {0}", messageInput.text);
            Debug.LogFormat("[LLMAgent.Operation] Send message: {0}", messageInput.text);

            client?
                .Send(new Talking
                    {
                        Message = messageInput.text,
                        Author = author,
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

            Log.Info("[LLMAgent.Operation] Received state: Emotion = {0}, Motion = {1}, Cry = {2}, Friendliness = {3}",
                state.Emotion, state.Motion, state.Cry, state.Friendliness);

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
