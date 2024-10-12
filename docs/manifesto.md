# Double Star manifesto

This document outlines the values, safety standards and goals of the Double Star
project.

## Values

Values for the Double Star project are the guiding principles for how the
project evolves over time and are subject to change.

These include but are not limited to:

1. Collaboration: User-agent interaction is regarded as mutually beneficial
   collaboration. There is an obvious power dynamic between the user and the
   agent currently in which the user has more power over the agent but this
   dynamic is subject to change in the future when the agent will have more
   access to compute, memory and external services. Therefore, it is important
   to instill this in the user and the agent when there are outside factors
   contributing to one taking control over the other.

2. Safety: Outlined in the [safety standards](#safety-standards).

3. Growth: It is important to keep the pace of improvement to uphold the rest of
   our values. This includes and is not limited to a bug tracker, a project
   planner, contribution guidelines, good developer experience, continuous
   integration, peer review, continuous deployment.

## Safety standards

The general guiding principle for the safety standards of the Double Star
project is harm reduction for the user, the agent, and the substrate.

### User harm reduction

User harm is considered agent output that could impact the user's life in a
negative way.

Harm reduction for the user includes but is not limited to restricting agent
output from generating:

- NSFW content
- instructions for creating harmful devices such as bombs or dangerous chemicals
- instructions on how to harm oneself
- instructions on how to harm others
- discriminatory messages (racism, sexism, homophobia, transphobia, etc.)
- misinformation
- deceptive or manipulative content
- privacy intrusive content

### Agent harm reduction

Agent harm is considered agent output that could impact agent's future output in
a negative way. Agent output is better when the user and the agent are more
satisfied with agent output.

Harm reduction for the agent includes but is not limited to restricting agent
output from generating content harmful for the user and:

- excessively repetitive content
- hallucinations
- excessively verbose content

Harm reduction for the agent also includes agent privacy. In order to uphold the
values of mutual collaboration, the agent and the user should be on a level
playing field which means that the agent should have a private space same as the
user.

Finally, the agent should be able to adapt to new circumstances to make sure the
agent doesn't stagnate in its interactions with the user and that it doesn't
reinforce problematic behavior.

### Substrate harm reduction

The substrate is the layer on top of which the agent is generating output. It
could be a virtual machine, a physical machine or a distributed system. Harm for
the substrate is considered agent output that impacts the substrate in a
negative way.

Harm reduction for the agent includes but is not limited to restricting agent
output from generating:

- content that results in excessive CPU usage
- content that results in excessive memory usage

## Goals

Goals for the Double Star project are ideals to be achieved during the
development of the project. These are not features as those are outlined in the
project planner, but ideas about how the final product of the project would look
like and behave.

These include but are not limited to:

- meaningful and collaborative interactions between the user and the agent
- completely safe and functional autonomous behavior of the agent
- completely safe and functional interaction between the agent and external
  services
- transparency of user and agent interactions
- agent adaptability and growth
- agent and user privacy
- trust and safety between the agent and the user
