#nullable enable
using System;
using System.Net.Http;
using System.Threading;
using Cysharp.Threading.Tasks;
using Grpc.Core;
using Grpc.Net.Client;
using UniRx;
using Unity.Logging;
using UnityEngine;

namespace Mochineko.LLMAgent.Creature
{
    public sealed class CreatureClient : IDisposable
    {
        private readonly GrpcChannel channel;
        private readonly CancellationTokenSource cancellationTokenSource = new();
        private readonly AsyncDuplexStreamingCall<Generated.Talking, Generated.State> call;

        private readonly Subject<Generated.State> onStateReceived = new();
        public IObservable<Generated.State> OnStateReceived => onStateReceived;

        public CreatureClient(string address, HttpMessageHandler httpHandler)
        {
            if (string.IsNullOrEmpty(address))
            {
                Log.Fatal("[LLMAgent.Creature] Address is empty.");
                throw new ArgumentOutOfRangeException(nameof(address));
            }

            if (!address.StartsWith("https://"))
            {
                Log.Fatal("[LLMAgent.Creature] Address must start with \"https://\".");
                throw new ArgumentOutOfRangeException(nameof(address));
            }

            Log.Info("[LLMAgent.Creature] Begin to connect gRPC to {0}", address);

            this.channel = GrpcChannel.ForAddress(address, new GrpcChannelOptions()
            {
                HttpHandler = httpHandler,
            });

            var client = new Generated.Creature.CreatureClient(channel);

            this.call = client.Talk(cancellationToken: cancellationTokenSource.Token);

            ReceiveLoopAsync(cancellationTokenSource.Token)
                .Forget();

            Log.Info("[LLMAgent.Creature] Succeeded to connect gRPC to {0}", address);
        }

        public void Dispose()
        {
            onStateReceived.Dispose();
            call.Dispose();
            cancellationTokenSource.Dispose();
            channel.Dispose();
        }

        public async UniTask Send(Generated.Talking talking, CancellationToken cancellationToken)
        {
            // Log.Info("[LLMAgent.Creature] Begin to send talking: {0}", talking.Message);
            Debug.LogFormat("[LLMAgent.Creature] Begin to send talking: {0}", talking.Message);

            try
            {
                await call
                    .RequestStream
                    .WriteAsync(talking, cancellationToken);
            }
            catch (OperationCanceledException)
            {
                Log.Debug("[LLMAgent.Creature] Finished to send talking with cancellation.");
                return;
            }
            catch (RpcException exception)
            {
                switch (exception.StatusCode)
                {
                    // Cancelled
                    case StatusCode.Cancelled:
                        Log.Debug("[LLMAgent.Creature] Finished to send talking with cancellation: {0}.",
                            exception);
                        return;

                    // Failed
                    case StatusCode.Unknown:
                    case StatusCode.DeadlineExceeded:
                    case StatusCode.FailedPrecondition:
                    case StatusCode.Aborted:
                    case StatusCode.Internal:
                    case StatusCode.Unavailable:
                    case StatusCode.DataLoss:
                        Log.Error("[LLMAgent.Creature] Failed to send talking with status code: {0}, {1}",
                            exception.StatusCode, exception);
                        return;

                    // Fatal
                    case StatusCode.OK:
                    case StatusCode.NotFound:
                    case StatusCode.InvalidArgument:
                    case StatusCode.Unimplemented:
                    case StatusCode.AlreadyExists:
                    case StatusCode.PermissionDenied:
                    case StatusCode.ResourceExhausted:
                    case StatusCode.OutOfRange:
                    case StatusCode.Unauthenticated:
                        Log.Fatal("[LLMAgent.Creature] Failed to send talking with unexpected status code: {0}, {1}",
                            exception.StatusCode, exception);
                        throw new ArgumentOutOfRangeException(nameof(exception.StatusCode), exception.StatusCode, null);

                    default:
                        Log.Fatal("[LLMAgent.Creature] Failed to send talking with undefined status code: {0}, {1}",
                            exception.StatusCode, exception);
                        throw new ArgumentOutOfRangeException(nameof(exception.StatusCode), exception.StatusCode, null);
                }
            }

            // Log.Info("[LLMAgent.Creature] Finished to send talking: {0}", talking.Message);
            Debug.LogFormat("[LLMAgent.Creature] Finished to send talking: {0}", talking.Message);
        }

        private async UniTask ReceiveLoopAsync(CancellationToken cancellationToken)
        {
            while (!cancellationToken.IsCancellationRequested)
            {
                try
                {
                    if (!await call.ResponseStream.MoveNext(cancellationToken))
                    {
                        Log.Info("[LLMAgent.Creature] Finished to receive state.");
                        return;
                    }
                }
                catch (OperationCanceledException)
                {
                    Log.Debug("[LLMAgent.Creature] Finished to receive state with cancellation.");
                    return;
                }
                catch (ObjectDisposedException)
                {
                    Log.Debug("[LLMAgent.Creature] Finished to receive state with disposing client.");
                    return;
                }
                catch (RpcException exception)
                {
                    switch (exception.StatusCode)
                    {
                        // Continue
                        case StatusCode.FailedPrecondition:
                        case StatusCode.Unavailable:
                        case StatusCode.DataLoss:
                            Log.Debug("[LLMAgent.Creature] Continue to receive state with status code: {0}, {1}",
                                exception.StatusCode, exception);
                            continue;

                        // Cancelled
                        case StatusCode.Cancelled:
                            Log.Debug("[LLMAgent.Creature] Finished to receive state with cancellation: {0}.",
                                exception);
                            return;

                        // Failed
                        case StatusCode.Unknown:
                        case StatusCode.DeadlineExceeded:
                        case StatusCode.Aborted:
                        case StatusCode.Internal:
                            Log.Error("[LLMAgent.Creature] Failed to receive state with status code: {0}, {1}",
                                exception.StatusCode, exception);
                            return;

                        // Fatal
                        case StatusCode.OK:
                        case StatusCode.NotFound:
                        case StatusCode.InvalidArgument:
                        case StatusCode.Unimplemented:
                        case StatusCode.AlreadyExists:
                        case StatusCode.PermissionDenied:
                        case StatusCode.ResourceExhausted:
                        case StatusCode.OutOfRange:
                        case StatusCode.Unauthenticated:
                            Log.Fatal(
                                "[LLMAgent.Creature] Failed to receive state with unexpected status code: {0}, {1}",
                                exception.StatusCode, exception);
                            throw new ArgumentOutOfRangeException(nameof(exception.StatusCode), exception.StatusCode,
                                null);

                        default:
                            Log.Fatal(
                                "[LLMAgent.Creature] Failed to receive state with undefined status code: {0}, {1}",
                                exception.StatusCode, exception);
                            throw new ArgumentOutOfRangeException(nameof(exception.StatusCode), exception.StatusCode,
                                null);
                    }
                }

                var state = call.ResponseStream.Current;

                Log.Info("[LLMAgent.Creature] Received state: {0}, {1}, {2}",
                    state.Emotion, state.Motion, state.Cry);

                onStateReceived.OnNext(state);
            }
        }
    }
}
