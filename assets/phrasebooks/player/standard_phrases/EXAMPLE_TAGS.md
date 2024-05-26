The following are examples on how we could encode the meaning of the phrases.

---

```yaml
## assets/phrasebooks/player/standard_phrases/farewells_hopeful.yaml

phrase: "We hope you find peace." 
semantic_tags: 
  - Farewell
  - Statement: Wish #  The player is expressing a wish for the ghost's well-being.
speech_act: Expressive # The phrase is primarily about expressing emotion.
phrase_vector:
  informal_formal: 0.5 #  Moderately informal, but not overly casual.
  aggressive_friendly: 0.8 # Very friendly and compassionate.
  threatening_reassuring: 0.9 # Highly reassuring, suggesting a desire for the ghost to feel better. 
  disrespectful_respectful: 0.6 #  Quite respectful, acknowledging the ghost's feelings. 
  apathetic_curious: -0.2 #  Not focused on curiosity, but on offering comfort. 
  submissive_dominant: -0.6 #  Quite submissive, showing empathy and willingness to help.
  honest_deceptive: 0.2 #  The phrase is likely sincere, but could be a way to dismiss the ghost.
  serious_humorous: -0.5 #  Moderately serious, conveying a sense of care.
contextual_tags: 
  # (Added at runtime) 

phrase: "We'll try to understand you better." 
semantic_tags: 
  - Farewell 
  - Statement: Intention # The player is stating their future actions. 
speech_act: Commissive # The player is making a promise. 
phrase_vector:
  informal_formal: 0.4 #  Slightly informal, but still polite.
  aggressive_friendly: 0.6 # Moderately friendly, suggesting a willingness to learn more. 
  threatening_reassuring: 0.7 #  Somewhat reassuring, suggesting no immediate threat.
  disrespectful_respectful: 0.4 #  Moderately respectful, acknowledging the ghost's worthiness of understanding.
  apathetic_curious: 0.8 #  The player is curious about the ghost.
  submissive_dominant: -0.5 #  Somewhat submissive, indicating a desire to connect.
  honest_deceptive: 0.0 #  Neutral in terms of honesty or deception. 
  serious_humorous: 0.1 #  Slightly serious, focusing on the task of understanding. 
contextual_tags: 
  # (Added at runtime) 

phrase: "May your spirit find rest." 
semantic_tags: 
  - Farewell
  - Wish: Peace # The player is expressing a wish for the ghost's peace.
speech_act: Expressive # The phrase is primarily about expressing emotion.
phrase_vector:
  informal_formal: 0.6 #  Moderately informal, but not overly casual.
  aggressive_friendly: 0.8 # Very friendly and compassionate.
  threatening_reassuring: 0.9 #  Highly reassuring, suggesting a desire for the ghost to feel better. 
  disrespectful_respectful: 0.6 #  Quite respectful, acknowledging the ghost's feelings. 
  apathetic_curious: -0.2 #  Not focused on curiosity, but on offering comfort. 
  submissive_dominant: -0.6 #  Quite submissive, showing empathy and willingness to help.
  honest_deceptive: 0.2 #  The phrase is likely sincere, but could be a way to dismiss the ghost.
  serious_humorous: -0.5 #  Moderately serious, conveying a sense of care.
contextual_tags: 
  # (Added at runtime) 

phrase: "We're sending you love and light." 
semantic_tags: 
  - Farewell 
  - Statement: Action #  The player is stating their action (sending positive energy).
speech_act: Commissive # The player is making a promise, although it's mostly symbolic.
phrase_vector:
  informal_formal: -0.4 #  Slightly informal, but still respectful. 
  aggressive_friendly: 0.8 #  Very friendly and non-aggressive. 
  threatening_reassuring: 0.9 #  Highly reassuring, suggesting a desire for the ghost to feel safe.
  disrespectful_respectful: 0.5 #  Moderately respectful, acknowledging the ghost's feelings.
  apathetic_curious: -0.1 #  Not focused on curiosity, but on offering comfort. 
  submissive_dominant: -0.7 #  Quite submissive, demonstrating a willingness to help.
  honest_deceptive: 0.0 #  Neutral in terms of honesty or deception. 
  serious_humorous: -0.4 #  Moderately serious, conveying a sense of genuine care.
contextual_tags: 
  # (Added at runtime) 

## assets/phrasebooks/player/standard_phrases/farewells_polite.yaml

phrase: "Goodbye."
semantic_tags:
  - Farewell
speech_act: Expressive # This is just a polite way of ending the interaction.
phrase_vector:
  informal_formal: 0.7 #  Moderately formal, but not overly stiff.
  aggressive_friendly: 0.5 #  Moderately friendly, but not overly effusive.
  threatening_reassuring: 0.3 #  Neutral, not threatening, but not explicitly reassuring either.
  disrespectful_respectful: 0.4 #  Moderately respectful, indicating politeness. 
  apathetic_curious: -0.4 #  Not curious, just ending the communication. 
  submissive_dominant: -0.1 #  Slightly submissive, being polite.
  honest_deceptive: 0.0 #  Neutral in terms of honesty or deception.
  serious_humorous: -0.1 #  Slightly serious, not playful or sarcastic.
contextual_tags: 
  # (Added at runtime) 

phrase: "Thank you for speaking with us."
semantic_tags: 
  - Farewell
  - Gratitude
speech_act: Expressive # This is primarily about expressing gratitude. 
phrase_vector:
  informal_formal: 0.6 #  Moderately formal, showing appreciation. 
  aggressive_friendly: 0.7 #  Moderately friendly, expressing goodwill.
  threatening_reassuring: 0.4 #  Slightly reassuring, suggesting no hostility. 
  disrespectful_respectful: 0.6 #  Quite respectful, showing gratitude.
  apathetic_curious: -0.2 #  Not curious, but expressing thanks.
  submissive_dominant: -0.3 #  Slightly submissive, showing deference.
  honest_deceptive: 0.0 #  Neutral in terms of honesty or deception.
  serious_humorous: -0.2 #  Moderately serious, expressing a sincere thank you. 
contextual_tags: 
  # (Added at runtime) 

phrase: "It was nice meeting you."
semantic_tags:
  - Farewell
  - Positive Experience
speech_act: Expressive # This is about expressing a positive feeling about the interaction.
phrase_vector:
  informal_formal: 0.4 #  Slightly informal, but still polite.
  aggressive_friendly: 0.8 # Very friendly, suggesting a positive interaction.
  threatening_reassuring: 0.5 #  Somewhat reassuring, suggesting no negativity.
  disrespectful_respectful: 0.5 #  Moderately respectful, acknowledging the ghost.
  apathetic_curious: -0.3 #  Not curious, just expressing a positive sentiment.
  submissive_dominant: -0.2 #  Slightly submissive, showing politeness. 
  honest_deceptive: 0.0 #  Neutral in terms of honesty or deception.
  serious_humorous: 0.1 #  Slightly playful, expressing a lighthearted sentiment.
contextual_tags: 
  # (Added at runtime) 

phrase: "We'll be back soon."
semantic_tags:
  - Farewell
  - Statement: Intention  # The player is stating their intention to return. 
speech_act: Commissive #  The player is making a promise. 
phrase_vector:
  informal_formal: -0.2 #  Slightly informal, but still respectful. 
  aggressive_friendly: 0.5 #  Moderately friendly, suggesting a willingness to continue the investigation. 
  threatening_reassuring: 0.5 #  Neutral, not threatening, but not explicitly reassuring either.
  disrespectful_respectful: 0.4 #  Moderately respectful, acknowledging the ghost.
  apathetic_curious: 0.6 #  The player is curious about the ghost's reaction and potentially future interactions. 
  submissive_dominant: -0.2 #  Slightly submissive, showing politeness.
  honest_deceptive: 0.0 #  Neutral in terms of honesty or deception.
  serious_humorous: 0.0 #  Neutral, neither playful nor overly serious.
contextual_tags:
  # (Added at runtime) 

## assets/phrasebooks/player/standard_phrases/farewells_urgent.yaml

phrase: "We need to leave now."
semantic_tags:
  - Farewell
  - Statement: Necessity # The player is stating an urgent need to depart.
speech_act: Assertive # The player is declaring their need to leave. 
phrase_vector:
  informal_formal: -0.2 #  Slightly informal, but still polite. 
  aggressive_friendly: 0.3 #  Moderately friendly, but not overly effusive.
  threatening_reassuring: -0.2 #  Slightly threatening, as it suggests a potential danger.
  disrespectful_respectful: 0.4 #  Moderately respectful, acknowledging the ghost. 
  apathetic_curious: -0.3 #  Not curious, just focused on leaving. 
  submissive_dominant: 0.2 #  Slightly dominant, asserting their need to depart.
  honest_deceptive: 0.0 #  Neutral in terms of honesty or deception.
  serious_humorous: -0.5 #  Moderately serious, conveying a sense of urgency.
contextual_tags: 
  # (Added at runtime)
  # Previous Interactions: Any (this does not depend on the previous interactions)

phrase: "We'll come back when it's safe."
semantic_tags:
  - Farewell
  - Statement: Intention # The player is stating their intention to return.
speech_act: Commissive # The player is making a promise to return later. 
phrase_vector:
  informal_formal: 0.1 #  Slightly informal, but still respectful. 
  aggressive_friendly: 0.4 #  Moderately friendly, suggesting a willingness to continue the investigation. 
  threatening_reassuring: 0.7 #  Moderately reassuring, suggesting no immediate threat. 
  disrespectful_respectful: 0.5 #  Moderately respectful, acknowledging the ghost. 
  apathetic_curious: 0.6 #  The player is curious about the ghost's reaction and potentially future interactions. 
  submissive_dominant: -0.3 #  Slightly submissive, showing politeness.
  honest_deceptive: 0.0 #  Neutral in terms of honesty or deception. 
  serious_humorous: -0.1 #  Slightly serious, focusing on the task of understanding. 
contextual_tags: 
  # (Added at runtime) 

## assets/phrasebooks/player/standard_phrases/greetings_curious.yaml

phrase: "Are you curious about us?"
semantic_tags: 
  - Question: Curiosity  #  The player is asking about the ghost's curiosity.
  - Relationship: Player-Ghost # The question is about the ghost's relationship with the player. 
speech_act: Directive #  The player is making a question, seeking a response.
phrase_vector:
  informal_formal: -0.4 #  Slightly informal, but still respectful.
  aggressive_friendly: 0.6 #  Moderately friendly, showing openness to communication.
  threatening_reassuring: 0.4 #  Slightly reassuring, suggesting no hostility.
  disrespectful_respectful: 0.5 #  Moderately respectful, acknowledging the ghost's feelings.
  apathetic_curious: 0.8 #  The player is curious about the ghost.
  submissive_dominant: 0.0 #  Neutral, not trying to assert dominance or submissiveness.
  honest_deceptive: 0.0 #  Neutral in terms of honesty or deception. 
  serious_humorous: 0.1 #  Slightly playful, expressing a lighthearted curiosity. 
contextual_tags: 
  # (Added at runtime) 

phrase: "Do you want to interact with us?"
semantic_tags: 
  - Question: Interaction # The player is asking about the ghost's desire to communicate.
  - Relationship: Player-Ghost # The question is about the ghost's relationship with the player. 
speech_act: Directive #  The player is making a question, seeking a response.
phrase_vector:
  informal_formal: -0.4 #  Slightly informal, but still respectful.
  aggressive_friendly: 0.6 #  Moderately friendly, showing openness to communication.
  threatening_reassuring: 0.4 #  Slightly reassuring, suggesting no hostility.
  disrespectful_respectful: 0.5 #  Moderately respectful, acknowledging the ghost's feelings.
  apathetic_curious: 0.8 #  The player is curious about the ghost.
  submissive_dominant: 0.0 #  Neutral, not trying to assert dominance or submissiveness.
  honest_deceptive: 0.0 #  Neutral in terms of honesty or deception. 
  serious_humorous: 0.1 #  Slightly playful, expressing a lighthearted curiosity. 
contextual_tags: 
  # (Added at runtime) 

phrase: "We're interested in learning more about you."
semantic_tags: 
  - Statement: Interest  #  The player is stating their interest in the ghost. 
  - Relationship: Player-Ghost # The statement is about the ghost's relationship with the player. 
speech_act: Assertive  #  The player is declaring their intentions. 
phrase_vector:
  informal_formal: 0.4 #  Slightly informal, but still respectful.
  aggressive_friendly: 0.8 # Very friendly, showing openness to communication.
  threatening_reassuring: 0.7 #  Moderately reassuring, suggesting no hostility.
  disrespectful_respectful: 0.5 #  Moderately respectful, acknowledging the ghost's feelings.
  apathetic_curious: 0.7 #  The player is curious about the ghost.
  submissive_dominant: -0.4 #  Slightly submissive, indicating a desire to connect.
  honest_deceptive: 0.0 #  Neutral in terms of honesty or deception. 
  serious_humorous: 0.1 #  Slightly playful, expressing a lighthearted curiosity. 
contextual_tags: 
  # (Added at runtime) 

phrase: "Can you tell us about yourself?"
semantic_tags: 
  - Question: Identity # The player is asking about the ghost's background. 
  - Relationship: Player-Ghost # The question is about the ghost's relationship with the player. 
speech_act: Directive # The player is making a question, seeking a response.
phrase_vector:
  informal_formal: -0.2 #  Slightly informal, but still respectful.
  aggressive_friendly: 0.6 # Moderately friendly, showing openness to communication.
  threatening_reassuring: 0.3 #  Neutral, not threatening, but not explicitly reassuring either.
  disrespectful_respectful: 0.5 #  Moderately respectful, acknowledging the ghost's feelings.
  apathetic_curious: 0.8 #  The player is curious about the ghost.
  submissive_dominant: 0.0 #  Neutral, not trying to assert dominance or submissiveness.
  honest_deceptive: 0.0 #  Neutral in terms of honesty or deception. 
  serious_humorous: 0.0 #  Neutral, neither playful nor overly serious.
contextual_tags: 
  # (Added at runtime) 

phrase: "What can you show us?"
semantic_tags: 
  - Question: Ability # The player is asking about the ghost's powers. 
  - Relationship: Player-Ghost # The question is about the ghost's relationship with the player. 
speech_act: Directive #  The player is making a question, seeking a response.
phrase_vector:
  informal_formal: -0.4 #  Slightly informal, but still respectful.
  aggressive_friendly: 0.5 #  Moderately friendly, showing openness to communication.
  threatening_reassuring: 0.3 #  Neutral, not threatening, but not explicitly reassuring either.
  disrespectful_respectful: 0.5 #  Moderately respectful, acknowledging the ghost's feelings.
  apathetic_curious: 0.8 #  The player is curious about the ghost.
  submissive_dominant: 0.0 #  Neutral, not trying to assert dominance or submissiveness.
  honest_deceptive: 0.0 #  Neutral in terms of honesty or deception. 
  serious_humorous: 0.0 #  Neutral, neither playful nor overly serious.
contextual_tags: 
  # (Added at runtime) 

## assets/phrasebooks/player/standard_phrases/greetings_formal.yaml

phrase: "Greetings."
semantic_tags:
  - Greeting 
speech_act: Expressive #  A simple greeting, expressing acknowledgment of the ghost's presence.
phrase_vector:
  informal_formal: 0.8 #  Formal greeting. 
  aggressive_friendly: 0.4 #  Neutral, not overly friendly or aggressive.
  threatening_reassuring: 0.2 #  Slightly reassuring, suggesting no hostility.
  disrespectful_respectful: 0.5 #  Moderately respectful, acknowledging the ghost's presence.
  apathetic_curious: 0.0 #  Not curious, just acknowledging the presence. 
  submissive_dominant: 0.0 #  Neutral, not trying to assert dominance or submissiveness.
  honest_deceptive: 0.0 #  Neutral in terms of honesty or deception.
  serious_humorous: -0.2 #  Slightly serious, not playful or sarcastic.
contextual_tags: 
  # (Added at runtime) 

phrase: "We are here to investigate."
semantic_tags: 
  - Statement: Intention #  The player is stating their purpose. 
  - Action: Investigate # The player is conducting an investigation.
speech_act: Assertive  #  The player is declaring their intentions.
phrase_vector:
  informal_formal: 0.6 #  Moderately formal, indicating a professional approach.
  aggressive_friendly: 0.3 #  Neutral, not overly friendly or aggressive.
  threatening_reassuring: 0.1 #  Slightly reassuring, but not overly calming.
  disrespectful_respectful: 0.4 #  Moderately respectful, acknowledging the ghost's presence. 
  apathetic_curious: 0.4 #  The player is curious about the ghost, but primarily focused on their investigation.
  submissive_dominant: 0.0 #  Neutral, not trying to assert dominance or submissiveness.
  honest_deceptive: 0.0 #  Neutral in terms of honesty or deception. 
  serious_humorous: -0.3 #  Moderately serious, focusing on the investigation.
contextual_tags: 
  # (Added at runtime) 

phrase: "We are paranormal researchers."
semantic_tags: 
  - Statement: Identity  #  The player is identifying themselves.
  - Occupation: Researcher # The player is stating their professional role.
speech_act: Assertive  #  The player is declaring their identity and role.
phrase_vector:
  informal_formal: 0.7 #  Moderately formal, establishing a professional persona.
  aggressive_friendly: 0.3 #  Neutral, not overly friendly or aggressive.
  threatening_reassuring: 0.1 #  Slightly reassuring, but not overly calming.
  disrespectful_respectful: 0.5 #  Moderately respectful, acknowledging the ghost's presence. 
  apathetic_curious: 0.3 #  The player is curious about the ghost, but primarily focused on their investigation.
  submissive_dominant: 0.0 #  Neutral, not trying to assert dominance or submissiveness.
  honest_deceptive: 0.0 #  Neutral in terms of honesty or deception. 
  serious_humorous: -0.4 #  Moderately serious, focusing on the investigation.
contextual_tags: 
  # (Added at runtime) 

phrase: "We are conducting a paranormal investigation."
semantic_tags: 
  - Statement: Action # The player is describing their current action.
  - Action: Investigate  # The player is conducting an investigation. 
speech_act: Assertive  #  The player is declaring their actions.
phrase_vector:
  informal_formal: 0.6 #  Moderately formal, indicating a professional approach.
  aggressive_friendly: 0.3 #  Neutral, not overly friendly or aggressive.
  threatening_reassuring: 0.1 #  Slightly reassuring, but not overly calming.
  disrespectful_respectful: 0.4 #  Moderately respectful, acknowledging the ghost's presence. 
  apathetic_curious: 0.4 #  The player is curious about the ghost, but primarily focused on their investigation.
  submissive_dominant: 0.0 #  Neutral, not trying to assert dominance or submissiveness.
  honest_deceptive: 0.0 #  Neutral in terms of honesty or deception. 
  serious_humorous: -0.3 #  Moderately serious, focusing on the investigation.
contextual_tags: 
  # (Added at runtime) 

phrase: "We seek to understand the phenomena occurring in this location."
semantic_tags: 
  - Statement: Intention #  The player is stating their purpose.
  - Action: Investigate  # The player is conducting an investigation.
  - Action: Understand  # The player is seeking to understand the situation. 
speech_act: Assertive  #  The player is declaring their intentions. 
phrase_vector:
  informal_formal: 0.6 #  Moderately formal, indicating a professional approach.
  aggressive_friendly: 0.2 #  Neutral, not overly friendly or aggressive.
  threatening_reassuring: 0.1 #  Slightly reassuring, but not overly calming.
  disrespectful_respectful: 0.4 #  Moderately respectful, acknowledging the ghost's presence. 
  apathetic_curious: 0.6 #  The player is curious about the ghost, but primarily focused on their investigation.
  submissive_dominant: 0.0 #  Neutral, not trying to assert dominance or submissiveness.
  honest_deceptive: 0.0 #  Neutral in terms of honesty or deception. 
  serious_humorous: -0.3 #  Moderately serious, focusing on the investigation.
contextual_tags: 
  # (Added at runtime) 

phrase: "We approach you with respect and curiosity."
semantic_tags: 
  - Statement: Approach # The player is describing their manner of interaction. 
  - Attitude: Respectful  # The player is emphasizing their respectful approach.
  - Attitude: Curious  #  The player is emphasizing their curious approach.
speech_act: Assertive  #  The player is declaring their intentions. 
phrase_vector:
  informal_formal: 0.6 #  Moderately formal, indicating a professional approach.
  aggressive_friendly: 0.5 #  Moderately friendly, showing openness to communication.
  threatening_reassuring: 0.3 #  Neutral, not threatening, but not explicitly reassuring either.
  disrespectful_respectful: 0.6 #  Quite respectful, acknowledging the ghost's worthiness of understanding.
  apathetic_curious: 0.7 #  The player is curious about the ghost.
  submissive_dominant: -0.4 #  Slightly submissive, indicating a desire to connect.
  honest_deceptive: 0.0 #  Neutral in terms of honesty or deception. 
  serious_humorous: 0.0 #  Neutral, neither playful nor overly serious.
contextual_tags: 
  # (Added at runtime) 

phrase: "We hope to establish communication with you."
semantic_tags: 
  - Statement: Intention # The player is stating their desire to communicate.
  - Relationship: Player-Ghost #  The statement is about the ghost's relationship with the player.
speech_act: Commissive #  The player is making a promise to try and communicate. 
phrase_vector:
  informal_formal: 0.4 #  Slightly informal, but still respectful.
  aggressive_friendly: 0.7 #  Moderately friendly, showing openness to communication.
  threatening_reassuring: 0.6 #  Moderately reassuring, suggesting no hostility.
  disrespectful_respectful: 0.5 #  Moderately respectful, acknowledging the ghost's feelings.
  apathetic_curious: 0.7 #  The player is curious about the ghost.
  submissive_dominant: -0.4 #  Slightly submissive, indicating a desire to connect.
  honest_deceptive: 0.0 #  Neutral in terms of honesty or deception. 
  serious_humorous: 0.0 #  Neutral, neither playful nor overly serious.
contextual_tags: 
  # (Added at runtime) 

## assets/phrasebooks/player/standard_phrases/greetings_friendly.yaml

phrase: "Hello?"
semantic_tags: 
  - Greeting
  - Question # The player is asking if the ghost is present.
speech_act: Directive # The player is making a question, seeking a response.
phrase_vector:
  informal_formal: -0.6 #  Fairly informal. 
  aggressive_friendly: 0.7 #  Moderately friendly, showing openness to communication.
  threatening_reassuring: 0.4 #  Slightly reassuring, suggesting no hostility. 
  disrespectful_respectful: 0.4 #  Moderately respectful, acknowledging the ghost's feelings.
  apathetic_curious: 0.6 #  The player is curious about the ghost.
  submissive_dominant: -0.1 #  Slightly submissive, showing politeness.
  honest_deceptive: 0.0 #  Neutral in terms of honesty or deception. 
  serious_humorous: 0.0 #  Neutral, neither playful nor overly serious.
contextual_tags: 
  # (Added at runtime) 

phrase: "Is anyone there?"
semantic_tags: 
  - Greeting 
  - Question # The player is asking if the ghost is present.
speech_act: Directive # The player is making a question, seeking a response.
phrase_vector:
  informal_formal: -0.4 #  Slightly informal, but still respectful.
  aggressive_friendly: 0.7 #  Moderately friendly, showing openness to communication.
  threatening_reassuring: 0.4 #  Slightly reassuring, suggesting no hostility. 
  disrespectful_respectful: 0.4 #  Moderately respectful, acknowledging the ghost's feelings.
  apathetic_curious: 0.6 #  The player is curious about the ghost.
  submissive_dominant: -0.1 #  Slightly submissive, showing politeness.
  honest_deceptive: 0.0 #  Neutral in terms of honesty or deception. 
  serious_humorous: 0.0 #  Neutral, neither playful nor overly serious.
contextual_tags: 
  # (Added at runtime) 

phrase: "We come in peace."
semantic_tags: 
  - Greeting # The phrase is intended as a greeting.
  - Statement: Intention  # The phrase is a statement about the player's intentions. 
speech_act: Assertive # The player is declaring their peaceful intentions. 
phrase_vector:
  informal_formal: 0.6 # Fairly formal tone. 
  aggressive_friendly: 0.8 # Highly friendly and non-aggressive.
  threatening_reassuring: 0.7 # Strongly reassuring and non-threatening.
  disrespectful_respectful: 0.4 # Moderately respectful.
  apathetic_curious: 0.2 # Slightly curious, but mostly focused on conveying a message.
  submissive_dominant: -0.5 # Somewhat submissive, indicating a willingness to cooperate. 
  honest_deceptive: 0.0 # Neutral in terms of honesty or deception. 
  serious_humorous: -0.2 # Slightly serious, but not overly somber.
contextual_tags: 
  # (Added at runtime) 

phrase: "We mean you no harm."
semantic_tags: 
  - Greeting # The phrase is intended as a greeting.
  - Statement: Intention  # The phrase is a statement about the player's intentions. 
speech_act: Assertive # The player is declaring their peaceful intentions. 
phrase_vector:
  informal_formal: 0.5 #  Moderately informal, but not overly casual.
  aggressive_friendly: 0.8 # Highly friendly and non-aggressive.
  threatening_reassuring: 0.7 # Strongly reassuring and non-threatening.
  disrespectful_respectful: 0.4 # Moderately respectful.
  apathetic_curious: 0.2 # Slightly curious, but mostly focused on conveying a message.
  submissive_dominant: -0.5 # Somewhat submissive, indicating a willingness to cooperate. 
  honest_deceptive: 0.0 # Neutral in terms of honesty or deception. 
  serious_humorous: -0.2 # Slightly serious, but not overly somber.
contextual_tags: 
  # (Added at runtime) 

```

