**I. Introduction**

Unhaunter is a 2D isometric game where players take on the role of paranormal
investigators, exploring haunted locations, gathering evidence, and ultimately
expelling restless spirits.  

Communication with these spirits is a crucial aspect of the gameplay experience.
It's not just about asking questions and getting answers; it's about building a
relationship with the ghost, understanding its motivations, and using your words
to influence its behavior.

This document outlines the vision and technical implementation of a dynamic and
emergent ghost communication system for Unhaunter.  This system aims to empower
players, create a sense of immersion, and add depth and variety to the gameplay.

**II.  Vision and Goals**

The goal of the ghost communication system is to create a truly engaging and
believable experience that goes beyond simple question-and-answer interactions. 
We want players to feel like they're having a real conversation with a complex
entity, not just following a script.

Here are the key goals of the system:

* **Emergent Language:**  Players should learn the ghost's "language" through 
  observation and experimentation.  The same phrase might trigger different 
  responses depending on the ghost's mood, the context of the conversation, 
  or other factors.
* **Dynamic Responses:**  The ghost's replies should be influenced by its mood, 
  the player's actions, and the environment.  This will create a sense of
  unpredictability and keep players engaged.
* **Emotional Depth:**  Ghosts should have distinct personalities and emotional
  states that affect their communication style.  Some ghosts might be shy and
  withdrawn, while others might be aggressive and confrontational.
* **Meaningful Choices:**  The player's communication choices should have
  consequences.  A well-chosen phrase might calm an agitated ghost, while a
  provocative statement might enrage it.

**III.  Design Principles**

The ghost communication system will be guided by the following design principles:

* **Data-Driven:**  The system will rely heavily on data to drive the ghost's 
   behavior and responses.  This data will include:
    * **FastText Embeddings:**  Word embeddings will be used to represent both 
      player phrases and ghost moods, allowing for semantic similarity
      comparisons.  This will enable the system to understand the meaning and 
      intent of phrases, even if they don't contain specific keywords.
    * **Hierarchical Tagging:**  Tags will be used to categorize phrases, filter
      options, and trigger specific responses.  This will allow for a more
      nuanced and flexible system than simple keyword matching.
    * **Ghost Mood Model:**  The ghost's emotional state will be represented by 
      a set of numerical values, each corresponding to a different emotion or
      state.  This mood will be dynamically updated based on player actions,
      environmental factors, and communication interactions.

* **Emergent Gameplay:**  The system will encourage players to learn the ghost's
  "language" through trial and error, observation, and experimentation.
  The same phrase might trigger different responses depending on the ghost's mood, 
  the context of the conversation, or other factors.
  This will create a sense of discovery and reward players for their understanding 
  of the ghost's nuances.

* **Contextual Relevance:**  The ghost's responses will be tailored to the 
  player's phrases, the conversation history, and the game's environment.
  This will make the communication feel more natural and immersive.  
  
  For example:
    * **Distance-Based Responses:**  The ghost's reactions might change based on
      the player's proximity.  A ghost might be more likely to whisper or make 
      subtle sounds when the player is far away, but it might become more vocal
      or aggressive when the player is close.
    * **Chain of Thought:**  The ghost's responses might evolve based on repeated
      questions or specific conversational patterns.  For example, if the player 
      repeatedly asks the ghost's name, it might become annoyed and refuse
      to answer, or it might even become more aggressive.

**IV.  Technical Implementation**

The ghost communication system will be implemented using a combination of data
structures, algorithms, and game logic.

Here's a breakdown of the key technical components:

* **Phrasebook Structure:**
    * **Hierarchical Tagging:**  The phrasebook will be organized using a 
      hierarchical tagging system.  This system will allow players to navigate the
      phrasebook and to find relevant phrases based on their intent and the context
      of the conversation.  For example, the phrasebook might have categories like
      "Greetings," "Questions," and "Requests," with subcategories
      like "Identity," "Motivation," and "Evidence."  
      Each phrase will be tagged with one or more tags that indicate 
      its meaning and purpose.
    * **YAML Format:**  The phrasebook data will be stored in YAML files, 
    which are human-readable and easy to edit.
    This will allow for easy customization and expansion of the phrasebook.
    * **Data Fields:**  Each phrase in the phrasebook will have the following data fields:
        * `phrase`:  The text of the phrase.
        * `id`:  A unique identifier for the phrase.
        * `tags`:  A list of tags associated with the phrase.
        * `repetition_count`:  A counter to track how many times the phrase has been used.
        * **(Optional):**  `response_triggers`, `emotional_weight`, `voiceover`, `translations`
    
* **Ghost Mood Model:**
    * **Data Structure:**  The ghost's mood will be represented by a `GhostMood` struct,
     which uses a `HashMap` to store the intensity of different emotions or states.
     The keys of the `HashMap` will be strings representing the emotions 
     (e.g., "fear", "anger"), and the values will be floating-point numbers 
     between 0.0 and 1.0, representing the intensity of each emotion.
    * **Mood Dimensions:**  The ghost mood model will include the following dimensions:
        * Fear
        * Anger
        * Sadness
        * Curiosity
        * Playfulness
        * **(Potentially more, depending on the desired complexity)**
    * **Mood Updates:**  The ghost's mood will be dynamically updated based on various factors:
        * **Player Actions:**  The player's proximity to the ghost, 
          the gear they use, and the noise they make can all influence the ghost's mood.
          For example, using an EMF reader near the ghost might increase its anger, 
          while speaking in a soothing tone might decrease its fear.
        * **Environmental Factors:**  The time of day, the temperature, 
          and the location within the haunted location can also affect the ghost's mood.
          For example, a ghost might be more active and aggressive at night, 
          or it might be more fearful in a particular room where it experienced a traumatic event.
        * **Ghost Type:**  Each ghost type will have an inherent temperament 
          that influences its baseline mood and how it reacts to different stimuli. 
          For example, a Banshee might be naturally more prone to anger, while a 
          Shade might be more inclined towards sadness and fear.
        * **Communication Interactions:**  The player's phrases and the ghost's 
          own responses can also affect its mood. 
          For example, a provocative question might increase the ghost's anger, 
          while a successful communication attempt might decrease its fear and increase its curiosity.

* **Response Templates:**
    * **YAML Format:**  The ghost response templates will be stored in a YAML file 
        named `ghost_responses.yaml`. 
        This file will contain a list of templates, each defining a possible ghost 
        response and the conditions under which it can be triggered.
    * **Template Structure:**  Each response template will have the following structure:
        ```yaml
        - trigger:
            # Trigger conditions
          response:
            # Response type and content
        ```
    * **Trigger Conditions:**  The `trigger` section of a template defines the
      conditions that must be met for the template to be considered.  These conditions can include:
        * `ghost_type`:  The type of ghost.  This allows for responses that are specific to certain ghost types.
        * `mood`:  A dictionary of emotions and their intensities.  This allows for responses that are triggered by specific emotional states.  For example, a template might only be triggered if the ghost's fear level is above 0.8.
        * `tags`:  A list of tags associated with the player's phrase.  This allows for responses that are relevant to the player's intent.  For example, a template might only be triggered if the player's phrase is tagged with "Question" and "Identity."
        * `distance`:  The distance between the player and the ghost.  This allows for responses that are appropriate for different distances.  For example, a ghost might be more likely to whisper or make subtle sounds when the player is far away, but it might become more vocal or aggressive when the player is close.
    * **Response Types:**  The `response` section of a template defines the type
       of response and its content.  The following response types will be supported:
        * `text`:  A textual response.  This could be a simple word, a phrase, or a complete sentence.
        * `action`:  An action performed by the ghost.  This could include actions like slamming a door, flickering a light, or throwing an object.
        * `event`:  A paranormal event triggered by the ghost.  This could include events like a temperature drop, a sudden gust of wind, or the appearance of an apparition.
        * `silence`:  The ghost remains silent.  This can be an effective way to create tension or to indicate that the ghost is not interested in communicating.
        * `sound`:  A sound effect played by the game.  This could be a whisper, a growl, a scream, or any other sound that enhances the atmosphere or conveys the ghost's emotional state.
    * **Response Content:**  The content of the response can be:
        * **Static Text:**  A pre-defined phrase or word that is always the same.
        * **Dynamic Text:**  Text that is generated at runtime based on the player's phrase, the ghost's mood, or other variables.  This can be achieved using placeholders in the response text.
        * **Instructions for Actions or Events:**  If the response type is `action` or `event`, the content will be an instruction for the game to perform the specified action or trigger the specified event.
    * **Placeholders:**  Placeholders can be used in the response text to create dynamic and personalized responses.  The following types of placeholders will be supported:
        * **Ghost Metadata:**  Placeholders that are replaced with values from
          the ghost's metadata.  For example, `[name]` would be replaced with 
          the ghost's name, `[profession]` would be replaced with a random profession
          from a list of possibilities, and `[loved one]` would be replaced with a random name
          from a list of loved ones.
        * **Player Information:**  Placeholders that are replaced with information
          about the player character.  For example, `[player name]` would be replaced
          with the player's name, and `[player action]` could be replaced with a
          description of the player's most recent action.
        * **Environmental Details:**  Placeholders that are replaced with details 
          about the game's environment.  For example, `[object name]` would be 
          replaced with the name of a nearby object, `[room name]` would be replaced 
          with the name of the current room, and `[time of day]` would be replaced 
          with the current time of day in the game.
* **Response Selection:**
    * **Matching Templates:**  When the player chooses a phrase, the system will
      first identify all the response templates that match the current situation.  
      This involves checking the ghost's type, mood, the tags associated with 
      the player's phrase, and the distance between the player and the ghost.
    * **Weighted Random Selection:**  If multiple templates match the current situation, 
      the system will use a weighted random selection process to choose a response.  
      The weights will be influenced by the ghost's mood and the relevance of the player's phrase.  
      For example, if the ghost is feeling fearful, templates that express fear will have a higher weight.  
      Similarly, if the player's phrase is a direct question about the ghost's identity, 
      templates that provide information about the ghost's name or backstory will have a higher weight.
* **Player AI (Future):**
    * **Concept:**  In the future, we plan to implement a player AI that will 
      track the player's emotional state and actions throughout the game.  
      This AI will be used to filter the dialogue options presented to the player, 
      ensuring that the choices are contextually relevant and in line with the 
      player's current understanding of the situation.
    * **Benefits:**  The player AI will create a more natural and intuitive communication experience.  
      It will prevent the player from seeing irrelevant or nonsensical dialogue options, 
      and it will encourage them to think strategically about their communication choices.

**V.  Tying It All Together**

The ghost communication system is a complex interplay of data, algorithms, 
and game logic, all working together to create a dynamic and emergent experience.  

Here's how the different components interact:

* **The Communication Loop:**  The communication between the player and the ghost is a cyclical process:
    1. **Player Action:**  The player performs an action in the game world, such as entering a room, using a piece of equipment, or making a noise.
    2. **Ghost Mood Update:**  The ghost's mood is updated based on the player's action, the environment, and its inherent temperament.
    3. **Player Phrase Selection:**  The player chooses a phrase from the phrasebook, guided by the bubble interface and the player AI's filtering (in the future).
    4. **Phrase Analysis:**  The system analyzes the player's chosen phrase, extracting its embedding, tags, and repetition count.
    5. **Response Template Matching:**  The system identifies applicable response templates based on the ghost's mood, the phrase's tags, and the distance between the player and the ghost.
    6. **Response Selection:**  A response template is selected using weighted random selection, with weights influenced by the ghost's mood and the phrase's relevance.
    7. **Response Generation:**  The response is generated based on the selected template, replacing placeholders with dynamic values from the ghost metadata, player information, and environment.
    8. **Ghost Response:**  The ghost delivers its response, which could be text, an action, an event, or silence.
    9. **Player Observation:**  The player observes the ghost's response and interprets its meaning, taking into account the context of the conversation and the ghost's behavior.
    10. **The Cycle Continues:**  The player chooses another phrase, and the loop repeats, with the ghost's mood and the conversation history influencing subsequent interactions.

* **Example Scenario:**  Let's imagine a scenario where the player encounters the Timid Shade in a dark hallway:

    1. **Player Action:**  The player enters the hallway and shines their flashlight around, illuminating the Shade's shadowy form.
    2. **Ghost Mood Update:**  The Shade's fear level increases due to the sudden light and the player's presence.
    3. **Player Phrase Selection:**  The player, noticing the Shade's apparent fear, chooses the phrase "We mean you no harm" from the "Reassurance" category of the phrasebook.
    4. **Phrase Analysis:**  The system analyzes the phrase's embedding, identifying its reassuring tone.  The tags associated with the phrase are "Statement" and "Reassurance."  The repetition count is 1, as this is the first time the player has used this phrase.
    5. **Response Template Matching:**  The system searches for response templates that match the Shade's ghost type, its current mood (high fear), the phrase's tags, and the distance between the player and the Shade (let's assume they are close).
    6. **Response Selection:**  Several templates might match the trigger conditions.  The system uses weighted random selection to choose a response, giving higher weights to templates that express fear or reassurance.  Let's say the selected template is:
        ```yaml
        - trigger:
            ghost_type: Shade
            mood:
              fear: 0.7
            tags: [Statement, Reassurance]
            distance: close
          response:
            type: text
            content: "Do you really mean that?"
        ```
    7. **Response Generation:**  The response is generated, and there are no placeholders to replace in this case.
    8. **Ghost Response:**  The Shade, in a trembling voice, asks, "Do you really mean that?"
    9. **Player Observation:**  The player hears the Shade's question and interprets it as a sign of vulnerability and a desire for reassurance.
    10. **The Cycle Continues:**  The player chooses another phrase, perhaps continuing to offer reassurance or asking a gentle question about the Shade's identity.  The Shade's mood and the conversation history will influence its response to the player's next choice.

**VI.  Current Progress**

The `fasttext_explorer` tool is a command-line utility written in Rust that allows us to prototype and test the ghost communication system.  It currently has the following capabilities:

* **Loading and Parsing the Phrasebook:**  The tool can load the player phrasebook from YAML files and parse it into a tree-like data structure that represents the hierarchical tagging system.
* **Generating and Storing Phrase Embeddings:**  The tool can use a pre-trained FastText model to generate embeddings for each phrase in the phrasebook and store them in JSONL files.
* **Loading Ghost Metadata:**  The tool can load ghost metadata from YAML files, including the ghost's type, initial mood, and any other relevant information.
* **Loading Response Templates:**  The tool can load response templates from the `ghost_responses.yaml` file.
* **Simulating Basic Ghost Responses:**  The tool can simulate basic ghost responses to player phrases, taking into account the ghost's type, mood, distance from the player, and the conversation history.  It can generate text responses, actions, events, and silence.

**Next Steps:**

* **Placeholder Replacement:**  Implement the logic to replace placeholders in response templates with dynamic values from the ghost metadata, player information, and environment.
* **Repetition Counting:**  Integrate repetition counting into the response selection logic to create more varied responses when a phrase is repeated.
* **Mood Updates:**  Develop a more sophisticated system for updating the ghost's mood based on player actions, environmental factors, and communication interactions.
* **Integration with Game Logic:**  Connect the communication system to the ghost AI, the player's actions, and the game's environment in the main Unhaunter game.

**VII.  Future Plans**

We have several exciting plans for enhancing the ghost communication system in the future:

* **Advanced Response Generation:**  We're exploring more advanced techniques for generating ghost responses, such as:
    * **The "Delta" Method:**  This method involves analyzing the differences (or "deltas") between question and answer embeddings to predict plausible answers to new questions.  This could allow the ghost to respond to a wider range of player queries in a more contextually relevant way.
    * **Emotional Weighting:**  We could assign emotional weights to phrases and tags, allowing the player's communication to have a more nuanced impact on the ghost's mood.  This would create a more dynamic and responsive system.
    * **Response Chains:**  We could use techniques like decision trees or Markov chains to create chains of responses that unfold over multiple turns.  This would allow for more complex and engaging conversations with the ghosts.
* **Player AI Development:**  We plan to develop a player AI that will track the player's emotional state and actions throughout the game.  This AI will be used to filter the dialogue options presented to the player, ensuring that the choices are contextually relevant and in line with the player's current understanding of the situation.  This will create a more natural and intuitive communication experience.
* **Voice Acting:**  We're considering adding voice acting to the ghost responses to further enhance the immersion and emotional impact of the communication.  We might explore using AI voice generation to create a wide range of voices and emotional tones.

**VIII.  Conclusion**

The ghost communication system is a central element of Unhaunter's gameplay, and we're committed to creating a system that is both innovative and engaging.  By combining FastText embeddings, hierarchical tagging, dynamic mood modeling, and data-driven response templates, we're building a system that allows for emergent, believable, and emotionally resonant interactions between players and ghosts.

We encourage contributors to explore the code, experiment with the `fasttext_explorer` tool, and share their ideas for improving the system.  Together, we can create a truly unique and unforgettable ghost hunting experience! 
