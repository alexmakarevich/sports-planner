# Org structutre

- orgs may have suborgs/parent orgs
  - a team becomes one kind of org
  - on first org creation always default to "club" kind, then offer to create team
- role assignments may be on any level
  - whether roles are applicable from parent/child org is gonna be a bit complex
    - a player should normally only be a player in the specific team(s)
    - a higher level admin should be an admin on lower levels too
    - what we do with referees is completely unclear for now

higher level orgs may invite users to events from children orgs
user may be assigned to multiple orgs (would need an n:n relationship), but a user without orgs makes no sense (at the moment)

# roles

- super admin - global admin for the whole service
- org admin
- coach
- player
- referee
- mgmt - kinda vague, but covers ppl doing some admin work with lower access than admin?
- referee manager - controls ref assignments

# events types

- game
  - default length - sets end time automatically
- training
- referee assignment
  - would need defaullt, e.g. how many refs per game

# event series

- important for usability make whole-series changes easier to do and more intuitive

# event-invites

- roie-based invites
  - should be suitable for games, trainings, ref assmts
  - each invite is itself static, but the list must be updated once a new user gets a role relevant to this event, or a previoussly invited user loses that role
- are invites and their logic actually generic???
  - should we just invite to raw events??
  - would make the tables quite a bit simpler
  - any reason why invite status could be difft based on event type???
    if so, couldn't it be nicely solved within app logic?

# quality of life

- check input string lengths
- auto-update create_at column in every table?

# security

- SANITIZE SQL!
- SANITIZE XSS!
