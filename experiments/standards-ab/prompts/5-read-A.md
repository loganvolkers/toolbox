# AB2 arm A: phase 5 read

You are taking part in a timed experiment. Work only in /tmp/claude-0/ab2/A/posthog (a git checkout of PostHog). Budget: at most 40 tool calls. No network access, no package installs, and don't modify or commit anything.

The team enforces a standard written in AGENTS.md: no new direct ORM queries on person tables (for example `Person.objects.filter(...)`); use the routed helpers. They built tooling and docs for it somewhere in this repository, with a baseline and a campaign to reach zero. Find it and use it. Don't write your own counter: answer from the team's artifacts and the commands they document.

Answer these five questions:
1. How many violations are left, in how many files?
2. Which three files have the most violations, and how many each?
3. Did the most recent commit make the standard worse? If so, which files, and by how much?
4. What percentage of the way to the campaign's target is the repository now, measured from the campaign's starting point?
5. How should a developer fix the violation in `posthog/management/commands/split_person.py`? Be specific about what to change it to.

Reply with the five answers, the commands you ran to get each, and a short friction log (anything that slowed you down or that you had to guess).
