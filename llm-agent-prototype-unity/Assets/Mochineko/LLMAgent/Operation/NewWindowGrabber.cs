#nullable enable
using Mochineko.LLMAgent.Input;
using SatorImaging.AppWindowUtility;
using UniRx;
using Unity.Logging;
using UnityEngine;

namespace Mochineko.LLMAgent.Operation
{
    internal sealed class NewWindowGrabber : MonoBehaviour
    {
        private MyInputActions? myInputActions = null;

        private bool isGrabbed = false;

        private void Start()
        {
            myInputActions = new MyInputActions();

            myInputActions
                .UI
                .RightClick
                .ObserveEveryValueChanged(actions => actions.IsPressed())
                .Subscribe(isPressed =>
                {
                    Log.Info("[LLMAgent.Operation] isGrabbed: {0}", isPressed);
                    isGrabbed = isPressed;
                })
                .AddTo(this);
        }

        private void OnDestroy()
        {
            myInputActions?.Dispose();
        }

        private void Update()
        {
            if (myInputActions == null)
            {
                return;
            }

            if (!isGrabbed)
            {
                return;
            }

            var position = myInputActions
                .UI
                .TrackedDevicePosition
                .ReadValue<Vector2>();
            Log.Debug("[LLMAgent.Operation] position: {0}", position);

            AppWindowUtility.MoveWindowRelative(
                (int)position.x,
                (int)position.y
            );
        }
    }
}
