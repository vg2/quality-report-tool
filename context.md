# Goal

I'd like to easily evaluate software quality in order to report on the state of quality to stakeholders, providing evidence based on software quality practices and measures in place ensuring high quality, and conversely where there's poor quality with measured evidence, and lastly on where there are gaps in processes and measures meaning assessing quality is difficult and not possible or incorrect. I would also like to suggest quality practice improvements - prioritising it based on where something to be added or changed would have the most positive impact on quality.

# Vision

I'd like to build a system that can be used to
assess the quality of another software solution. The criteria on which it is evaluated should be based on the **ISO/IEC 25010** (Systems and Software Quality Requirements and Evaluation) standard.

I'd like to provide context of the system (structured information) that would detail the type of system using likely a number (3+) of characteristics (e.g. customer facing, transactional or employee facing, reporting or backend data processing, etc). I could capture this "manually" or provide context in natural language and the system can infer the structured types it supports.

Based on the type of system there would be a catalog of quality measures and practices that would be based on the ISO quality model mentioned in goal, categorised by characteristic and sub characteristic of the model.

The idea would be me as the end user (solutions architect / technical lead) could use this to provide additional context on the state of quality (what measures are already in place, and the results, e.g. performance -> response time, and attach a load test result) for as many areas as available to provide, and the system provides an overall state of quality with the ability to dive into each quality characteristic and sub characteristic for more detailed information.

The overall quality should give some sort of grading or rating of quality, as well as for each characteristic, with some sort of heat map to identify quickly hot spots of poor or lacking details of quality balanced with how important it is for that type of system.

The system should by default be able to store all of this in its own DB and be self hostable. It should be extensible such that it could be connected to other databases in the future. It should even be able to store it's data in things like Jira or azure DevOps boards to track the data, but this would be a future phase goal.

In principle, this system should only store information on quality and that references back to source systems and data from the enterprise, and only fill in those gaps if none exist. It should be allow plugging in sources of various types, such as git repositories, pipeline runs with test evidence, links to documentation / wiki's, etc. This could all be used as context to assess the quality without duplicating any of this existing information (temporary copies / cloned repositories are fine).

The system should be able to provide a polished PDF report of the quality of the system, as well as generate a polished presentation to showcase the state of quality. It should be white labelled to include your own company identity (assessor) and client company, or optionally just one identity throughout for company self assessment.

Ideally (and maybe second phase) you could connect it to live feeds of data (e.g. a performance test pipeline) so it always has up to date information on quality.

# Technology suggestions

- High performance cross platform desktop app - leaning towards Tauri based in Rust
- General Inference - I'd like to enable adherence to the company standards in which this tool is used, and use an inference provider of their choosing. This could be using authenticated cli tools like claude code, or APIs to AWS BedRock as examples.
- PDF reporting - To unpack as part of up front analysis
- Presentation generation - To unpack as part of un front analysis

# Security Concerns

- all major typical desktop app security measures should be in place above and beyond the below specifically called out areas
- customer IP should be protected from inference provider training and as much data as possible should be anonymised and generalised
- AI features should be protected from prompt injection and other similar attacks
