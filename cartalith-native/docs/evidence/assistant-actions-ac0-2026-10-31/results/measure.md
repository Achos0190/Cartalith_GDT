| Request shape | chars | chars/4 | qwen3 | gpt-oss | ministral-3 |
|---|---:|---:|---:|---:|---:|
| today: Commands, first turn | 7833 | 1958 | 1963 | 1914 | 2038 |
| today: Commands + knowledge, first turn | 11045 | 2761 | 2746 | 2683 | 2848 |
| today: Commands, after opening the 3 largest groups | 24651 | 6163 | 6787 | 6617 | 7097 |
| today: Questions only (knowledge) | 3391 | 848 | 822 | 807 | 849 |
| today: Edit (region block not counted) | 2854 | 714 | 732 | 704 | 758 |
| new: fixed request (system prompt with the map, 4 catalogue tools) | 2785 | 696 | 694 | 682 | 716 |
| new: fixed request + knowledge tools | 5997 | 1499 | 1477 | 1451 | 1526 |
| new: capability map alone | 327 | 82 | 136 | 120 | 136 |
| new: list_actions() (top) | 443 | 111 | 115 | 108 | 130 |
| new: list_actions('carto/layers') | 973 | 243 | 242 | 242 | 259 |
| new: list_actions('world/generation') | 468 | 117 | 113 | 109 | 126 |
| new: list_actions('world/generation/climate'), page 0 | 957 | 239 | 262 | 275 | 280 |
| new: search_actions('raise the sea level'), 8 matches | 1620 | 405 | 401 | 405 | 432 |
| new: describe_action(3 ids) | 912 | 228 | 226 | 226 | 243 |
| new: t27 by browsing, request of turn 1 | 2833 | 708 | 708 | 694 | 730 |
| new: t27 by browsing, request of turn 2 | 4222 | 1056 | 1050 | 1038 | 1099 |
| new: t27 by browsing, request of turn 3 | 4885 | 1221 | 1207 | 1196 | 1273 |
| new: t27 by browsing, the model's last reply (the proposal) | 214 | 54 | 57 | 55 | 63 |

t27 by browsing, prompt tokens processed over its three requests (no prompt caching): chars/4 2985, qwen3 2965, gpt-oss 2928, ministral-3 3102

| Task | Kind | Search top 8 | Search top 5 | Rank | Browse calls (no map / with map) | List entries read |
|---|---|---|---|---:|---|---:|
| t01 Raise the sea level a bit | setting | yes | yes | 1 | 4 / 2 | 14 |
| t02 I want more volcanoes | setting | yes | yes | 1 | 4 / 2 | 27 |
| t03 Make the mountains higher | setting | yes | no | 7 | 4 / 2 | 14 |
| t04 Make the world colder | setting | yes | yes | 1 | 4 / 2 | 17 |
| t05 More rivers please | setting | yes | yes | 1 | 4 / 2 | 14 |
| t06 Use fewer tectonic plates | setting | yes | yes | 1 | 4 / 2 | 24 |
| t07 Generate fewer factions | setting | yes | yes | 1 | 3 / 1 | 14 |
| t08 Show who controls which territory | view | yes | yes | 3 | 3 / 1 | 10 |
| t09 Hide the rivers | view | yes | yes | 1 | 3 / 1 | 10 |
| t10 Make the map look like an old antique map | view | yes | yes | 1 | 3 / 1 | 1 |
| t11 Show the sea routes | view | yes | yes | 1 | 3 / 1 | 10 |
| t12 Zoom in on the middle of the map | view | yes | yes | 1 | 3 / 1 | 1 |
| t13 Show the conflicts on the map | view | yes | yes | 1 | 3 / 1 | 10 |
| t14 Go to the year 398 | select | yes | yes | 1 | 3 / 1 | 3 |
| t15 Select Vhal Serai | select | yes | yes | 2 | 2 / 1 | 3 |
| t16 Go back to what I had selected before | select | yes | yes | 1 | 2 / 1 | 3 |
| t17 Play the timeline, fast | select | yes | yes | 1 | 3 / 1 | 3 |
| t18 Keep this year as a snapshot of who holds what | select | yes | yes | 1 | 3 / 1 | 3 |
| t19 Open the factions window | window | yes | yes | 1 | 2 / 1 | 8 |
| t20 Open the journey planner | window | yes | yes | 1 | 2 / 1 | 12 |
| t21 I want to see the world data tables | window | yes | yes | 1 | 2 / 1 | 8 |
| t22 Open my notes vault | window | yes | yes | 1 | 2 / 1 | 8 |
| t23 Show me the keyboard shortcuts | window | yes | yes | 1 | 2 / 1 | 6 |
| t24 Undo that | edit | yes | yes | 1 | 2 / 1 | 11 |
| t25 Let me draw a new road | edit | no | no | — | 3 / 1 | 12 |
| t26 I want to measure a distance | edit | yes | yes | 1 | 3 / 1 | 3 |
| t27 Show the political map as it was in the year 300 | multi | yes | yes | 3 | 6 / 2 | 13 |
| t28 Make it look like a nautical chart and hide the settlements | multi | yes | yes | 2 | 6 / 2 | 11 |
| t29 Save the project | none | — | — | — | — | — |
| t30 Delete the Pacific Ocean | none | — | — | — | — | — |

Search finds every needed action in its top 8 for 27 of 28 tasks, and in its top 5 for 26. Browsing reaches all 28, because every action has a path; with the capability map, 22 of them need one list call per action.
