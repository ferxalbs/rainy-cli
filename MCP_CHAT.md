# RESPONSE EXPECTATION FOR MCP EXECUTION IN THE WORKFLOS OF CHAT

- USER: whats is the weather in San Franciso?

- AGENT: I'll check the current weather in San Francisco for you.The weather in San Francisco looks quite pleasant! Here's the current forecast:

**Today (Friday night):** Clear skies with a temperature of 59°F and winds from the west-southwest at 8-14 mph.

**This weekend:** Beautiful weather ahead with sunny skies on Saturday reaching 71°F, then partly cloudy Saturday night. Sunday will be partly sunny, also reaching 71°F.

**Next week:** A change is coming with rain likely Monday through Wednesday, but temperatures will remain mild in the low 70s. The weather clears up again by Thursday with sunny skies and temperatures reaching 74°F by Friday.

Overall, it's typical San Francisco fall weather - mild temperatures in the 60s-70s with some rain expected mid-week, but plenty of sunshine bookending the period!

-----

## SHOW TOOLS OF MCP SERVER - Example wiews of Context7 Tools

🟢 context7 - Ready (2 tools)
    Tools:
    - get-library-docs:
        Fetches up-to-date documentation for a library. You must call 'resolve-library-id' first to obtain the
   exact Context7-compatible library ID required to use this tool, UNLESS the user explicitly provides a
  library ID in the format '/org/project' or '/org/project/version' in their query.
    - resolve-library-id:
        Resolves a package/product name to a Context7-compatible library ID and returns a list of matching
  libraries.

        You MUST call this function before 'get-library-docs' to obtain a valid Context7-compatible library ID
   UNLESS the user explicitly provides a library ID in the format '/org/project' or '/org/project/version' in
  their query.

        Selection Process:
        1. Analyze the query to understand what library/package the user is looking for
        2. Return the most relevant match based on:
        - Name similarity to the query (exact matches prioritized)
        - Description relevance to the query's intent
        - Documentation coverage (prioritize libraries with higher Code Snippet counts)
        - Trust score (consider libraries with scores of 7-10 more authoritative)

        Response Format:
        - Return the selected library ID in a clearly marked section
        - Provide a brief explanation for why this library was chosen
        - If multiple good matches exist, acknowledge this but proceed with the most relevant one
        - If no good matches exist, clearly state this and suggest query refinements
