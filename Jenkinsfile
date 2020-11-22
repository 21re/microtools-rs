@Library("21re") _

gen.init()

reportSlack {
  node {
    checkout scm

    rustBuild([version: gen.VERSION])

    sh("env && echo 'TEST'")
	
    if(gen.deploy) {
      githubRelease([version: gen.VERSION])
    }
  }

  if(gen.deploy) {
  }
}
