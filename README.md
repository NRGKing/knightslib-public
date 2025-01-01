
# Knights Lib
A PROS library for use with the V5RC system. This library is designed to help set up teams with a powerful control system to build their code upon. It features advanced systems like odometry and path following, as well as cosmetic features like a custom brain display. 

> This library is still in development and being tested, we will update the features list with the completion status of each object.

### Features
We will list our features with the format: `feature | completeness`
- Control Algorithms
	- Path Following
		- Pure Pursuit | Fully Complete
	- Position Tracking
		- Odometry | Fully Complete
			- Features many different configurations for tracking wheels and IMUs
	- Move To Point
		- Turn to Heading | Fully Complete
		- Move to Point | Coded
	- PID
	    - Lateral Movement | Fully Complete
	    - Turning Movement | Fully Complete
- Systems
	- Drivetrains
		- Tank/Differential | Fully Complete
- Cosmetics
	- Autonomous Selector | Fully Complete
	- Odometry Visual Display | Fully Complete
- Route Generation
	- Read basic route from SD Card | Fully Complete
	- Read advanced route from SD Card | Fully Complete

### Docs & Tutorials
Work in progress, if you have questions, please message me on discord. (Username: nrgking)

### Contribution
> **Legal Note About Contributing**
> If you contribute to this library, you agree that you have created 100% of your content and have rights to it, and that you understand that your content will be provided under the license of the project.

If you would like to contribute, please fork the project and propose your changes. Eventually we will release a style guide and a contribution guide, but here are a few general guidelines:

#### 1. New Features
If you have any ideas or requests for features, please create an enhancement request in the GitHub repository. If you would like to implement this, you are welcome to. However, even if you can't, we will still consider feature requests. This does mean that we might have to reject some features that don't fit with the project.
#### 2. Bugs
If you find any bugs with the library, please open an issue on GitHub.
#### 3.  Commenting
Please comment all functions in header files with doxygen comments. Code should also be readable without comments (descriptive variable names, etc.). For a more detailed guide, follow one such as [this](https://stackoverflow.blog/2021/12/23/best-practices-for-writing-code-comments/). 

Thank you for considering contributing to our library! Most of the current work has been done by a single team, so any help is appreciated.

