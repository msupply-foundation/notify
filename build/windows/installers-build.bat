@ECHO ##### Removing installers folder #####
@rmdir "installers" /s /q

@ECHO ##### Adjusting SUFS #####
FOR /F "delims=*" %%i in ('more notify\version.txt') do SET versionTag=%%i
@ECHO "current tag = %versionTag%"
SET installersOutputFolder=%WORKSPACE%\installers

@cd notify
node "adjustSUFs.js"
@cd ..

@ECHO ##### Creating installers #####
start "" /wait "C:\Program Files (x86)\Setup Factory 9\SUFDesign.exe" /BUILD /LOG:installers\setup-factory.log "notify\notify_service.suf"