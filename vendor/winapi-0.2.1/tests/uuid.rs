// Copyright © 2015, Peter Atashian
// Licensed under the MIT License <LICENSE.md>
#![cfg(windows)]
extern crate uuid;
use uuid::*;
#[inline(never)] fn bb<T>(_: T) {}
#[test]
fn statics() {
    bb(FOLDERID_AccountPictures);
    bb(FOLDERID_AddNewPrograms);
    bb(FOLDERID_AdminTools);
    bb(FOLDERID_AppUpdates);
    bb(FOLDERID_ApplicationShortcuts);
    bb(FOLDERID_AppsFolder);
    bb(FOLDERID_CDBurning);
    // bb(FOLDERID_CameraRoll);
    bb(FOLDERID_ChangeRemovePrograms);
    bb(FOLDERID_CommonAdminTools);
    bb(FOLDERID_CommonOEMLinks);
    bb(FOLDERID_CommonPrograms);
    bb(FOLDERID_CommonStartMenu);
    bb(FOLDERID_CommonStartup);
    bb(FOLDERID_CommonTemplates);
    bb(FOLDERID_ComputerFolder);
    bb(FOLDERID_ConflictFolder);
    bb(FOLDERID_ConnectionsFolder);
    bb(FOLDERID_Contacts);
    bb(FOLDERID_ControlPanelFolder);
    bb(FOLDERID_Cookies);
    bb(FOLDERID_Desktop);
    bb(FOLDERID_DeviceMetadataStore);
    bb(FOLDERID_Documents);
    bb(FOLDERID_DocumentsLibrary);
    bb(FOLDERID_Downloads);
    bb(FOLDERID_Favorites);
    bb(FOLDERID_Fonts);
    bb(FOLDERID_GameTasks);
    bb(FOLDERID_Games);
    bb(FOLDERID_History);
    bb(FOLDERID_HomeGroup);
    bb(FOLDERID_HomeGroupCurrentUser);
    bb(FOLDERID_ImplicitAppShortcuts);
    bb(FOLDERID_InternetCache);
    bb(FOLDERID_InternetFolder);
    bb(FOLDERID_Libraries);
    bb(FOLDERID_Links);
    bb(FOLDERID_LocalAppData);
    bb(FOLDERID_LocalAppDataLow);
    bb(FOLDERID_LocalizedResourcesDir);
    bb(FOLDERID_Music);
    bb(FOLDERID_MusicLibrary);
    bb(FOLDERID_NetHood);
    bb(FOLDERID_NetworkFolder);
    bb(FOLDERID_OriginalImages);
    bb(FOLDERID_PhotoAlbums);
    bb(FOLDERID_Pictures);
    bb(FOLDERID_PicturesLibrary);
    bb(FOLDERID_Playlists);
    bb(FOLDERID_PrintHood);
    bb(FOLDERID_PrintersFolder);
    bb(FOLDERID_Profile);
    bb(FOLDERID_ProgramData);
    bb(FOLDERID_ProgramFiles);
    bb(FOLDERID_ProgramFilesCommon);
    bb(FOLDERID_ProgramFilesCommonX64);
    bb(FOLDERID_ProgramFilesCommonX86);
    bb(FOLDERID_ProgramFilesX64);
    bb(FOLDERID_ProgramFilesX86);
    bb(FOLDERID_Programs);
    bb(FOLDERID_Public);
    bb(FOLDERID_PublicDesktop);
    bb(FOLDERID_PublicDocuments);
    bb(FOLDERID_PublicDownloads);
    bb(FOLDERID_PublicGameTasks);
    bb(FOLDERID_PublicLibraries);
    bb(FOLDERID_PublicMusic);
    bb(FOLDERID_PublicPictures);
    bb(FOLDERID_PublicRingtones);
    bb(FOLDERID_PublicUserTiles);
    bb(FOLDERID_PublicVideos);
    bb(FOLDERID_QuickLaunch);
    bb(FOLDERID_Recent);
    bb(FOLDERID_RecordedTVLibrary);
    bb(FOLDERID_RecycleBinFolder);
    bb(FOLDERID_ResourceDir);
    bb(FOLDERID_Ringtones);
    bb(FOLDERID_RoamedTileImages);
    bb(FOLDERID_RoamingAppData);
    bb(FOLDERID_RoamingTiles);
    bb(FOLDERID_SEARCH_CSC);
    bb(FOLDERID_SEARCH_MAPI);
    bb(FOLDERID_SampleMusic);
    bb(FOLDERID_SamplePictures);
    bb(FOLDERID_SamplePlaylists);
    bb(FOLDERID_SampleVideos);
    bb(FOLDERID_SavedGames);
    bb(FOLDERID_SavedSearches);
    bb(FOLDERID_Screenshots);
    // bb(FOLDERID_SearchHistory);
    bb(FOLDERID_SearchHome);
    // bb(FOLDERID_SearchTemplates);
    bb(FOLDERID_SendTo);
    bb(FOLDERID_SidebarDefaultParts);
    bb(FOLDERID_SidebarParts);
    // bb(FOLDERID_SkyDrive);
    // bb(FOLDERID_SkyDriveCameraRoll);
    // bb(FOLDERID_SkyDriveDocuments);
    // bb(FOLDERID_SkyDriveMusic);
    // bb(FOLDERID_SkyDrivePictures);
    bb(FOLDERID_StartMenu);
    bb(FOLDERID_Startup);
    bb(FOLDERID_SyncManagerFolder);
    bb(FOLDERID_SyncResultsFolder);
    bb(FOLDERID_SyncSetupFolder);
    bb(FOLDERID_System);
    bb(FOLDERID_SystemX86);
    bb(FOLDERID_Templates);
    bb(FOLDERID_UserPinned);
    bb(FOLDERID_UserProfiles);
    bb(FOLDERID_UserProgramFiles);
    bb(FOLDERID_UserProgramFilesCommon);
    bb(FOLDERID_UsersFiles);
    bb(FOLDERID_UsersLibraries);
    bb(FOLDERID_Videos);
    bb(FOLDERID_VideosLibrary);
    bb(FOLDERID_Windows);
}
