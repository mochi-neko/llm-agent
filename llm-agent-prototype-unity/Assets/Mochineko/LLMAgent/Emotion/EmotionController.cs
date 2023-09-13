#nullable enable
using System;
using UnityEngine;

namespace Mochineko.LLMAgent.Emotion
{
    public sealed class EmotionController : MonoBehaviour
    {
        [SerializeField]
        private GameObject? happy = null;

        [SerializeField]
        private GameObject? sad = null;

        [SerializeField]
        private GameObject? angry = null;

        [SerializeField]
        private GameObject? fearful = null;

        [SerializeField]
        private GameObject? disgusted = null;

        [SerializeField]
        private GameObject? surprised = null;

        public void SetEmotion(Creature.Generated.Emotion emotion)
        {
            if (happy == null)
            {
                throw new NullReferenceException(nameof(happy));
            }

            if (sad == null)
            {
                throw new NullReferenceException(nameof(sad));
            }

            if (angry == null)
            {
                throw new NullReferenceException(nameof(angry));
            }

            if (fearful == null)
            {
                throw new NullReferenceException(nameof(fearful));
            }

            if (disgusted == null)
            {
                throw new NullReferenceException(nameof(disgusted));
            }

            if (surprised == null)
            {
                throw new NullReferenceException(nameof(surprised));
            }

            switch (emotion)
            {
                case Creature.Generated.Emotion.Neutral:
                    happy.SetActive(false);
                    sad.SetActive(false);
                    angry.SetActive(false);
                    fearful.SetActive(false);
                    disgusted.SetActive(false);
                    surprised.SetActive(false);
                    break;

                case Creature.Generated.Emotion.Happy:
                    happy.SetActive(true);
                    sad.SetActive(false);
                    angry.SetActive(false);
                    fearful.SetActive(false);
                    disgusted.SetActive(false);
                    surprised.SetActive(false);
                    break;

                case Creature.Generated.Emotion.Sad:
                    happy.SetActive(false);
                    sad.SetActive(true);
                    angry.SetActive(false);
                    fearful.SetActive(false);
                    disgusted.SetActive(false);
                    surprised.SetActive(false);
                    break;

                case Creature.Generated.Emotion.Angry:
                    happy.SetActive(false);
                    sad.SetActive(false);
                    angry.SetActive(true);
                    fearful.SetActive(false);
                    disgusted.SetActive(false);
                    surprised.SetActive(false);
                    break;

                case Creature.Generated.Emotion.Fearful:
                    happy.SetActive(false);
                    sad.SetActive(false);
                    angry.SetActive(false);
                    fearful.SetActive(true);
                    disgusted.SetActive(false);
                    surprised.SetActive(false);
                    break;

                case Creature.Generated.Emotion.Disgusted:
                    happy.SetActive(false);
                    sad.SetActive(false);
                    angry.SetActive(false);
                    fearful.SetActive(false);
                    disgusted.SetActive(true);
                    surprised.SetActive(false);
                    break;

                case Creature.Generated.Emotion.Surprised:
                    happy.SetActive(false);
                    sad.SetActive(false);
                    angry.SetActive(false);
                    fearful.SetActive(false);
                    disgusted.SetActive(false);
                    surprised.SetActive(true);
                    break;

                default:
                    throw new ArgumentOutOfRangeException(nameof(emotion), emotion, null);
            }
        }
    }
}
