<script lang="js">
  import Confirm from './common/Confirm.svelte';
  import { dndzone } from 'svelte-dnd-action';

  import {
    CancelPlaylistDownload,
    GetPlaylistFromGdrive,
    PlayFileFromPlaylist,
    RestartPlaylistDownload,
    SetPlaylist,
    StopPlayingFileFromPlaylist,
  } from '../../api/client.graphql';
  import { mutation } from 'svelte-apollo';

  import Fa from 'svelte-fa';
  import {
    faDownload,
    faTriangleExclamation,
    faCirclePlay,
    faCircleStop,
  } from '@fortawesome/free-solid-svg-icons';

  import FileInfo from './common/FileInfo.svelte';

  import {
    getFileIdFromGDrive,
    getFolderIdFromGDrive,
    isFullGDrivePath,
    showError,
  } from '../utils/util';
  import PlaylistStatus from './common/PlaylistStatus.svelte';

  const restartPlaylistDownload = mutation(RestartPlaylistDownload);
  const cancelPlaylistDownload = mutation(CancelPlaylistDownload);
  const getPlaylistFromDrive = mutation(GetPlaylistFromGdrive);
  const setPlaylist = mutation(SetPlaylist);
  const playFileFromPlaylist = mutation(PlayFileFromPlaylist);
  const stopPlayingFileFromPlaylist = mutation(StopPlayingFileFromPlaylist);

  export let restreamId;
  export let queue;
  export let currentlyPlayingFileId;

  $: dragDisabled = true;

  $: hasDownloadingFiles = Boolean(queue.find((x) => x.isDownloading));

  $: hasFilesInPlaylist = Boolean(queue?.length > 0);

  let googleDriveFolderOrFileId = '';
  let isValidFolderIdInput = true;

  async function loadPlaylistOrFile(file_or_folder_id) {
    if (isValidFolderIdInput) {
      const variables = { restreamId, file_or_folder_id };
      try {
        await getPlaylistFromDrive({ variables });
        googleDriveFolderOrFileId = '';
      } catch (e) {
        showError(e.message);
      }
    } else {
      showError(
        `Google drive folder Id of file Id: ${file_or_folder_id} is incorrect`
      );
    }
  }

  async function stopPlaylistDownload() {
    try {
      const variables = { id: restreamId };
      await cancelPlaylistDownload({ variables });
    } catch (e) {
      showError(e.message);
    }
  }

  async function startPlaylistDownload() {
    try {
      const variables = { id: restreamId };
      await restartPlaylistDownload({ variables });
    } catch (e) {
      showError(e.message);
    }
  }

  async function clearPlaylist() {
    const variables = { restreamId, fileIds: [] };
    try {
      await setPlaylist({ variables });
    } catch (e) {
      showError(e.message);
    }
  }

  function getGDriveFileOrFolderId(id) {
    let fileOrFolderId = '';

    if (isFullGDrivePath(id)) {
      fileOrFolderId = getFolderIdFromGDrive(id);

      return isFullGDrivePath(fileOrFolderId)
        ? getFileIdFromGDrive(id)
        : fileOrFolderId;
    }

    return id;
  }

  function handleInputFolderId(event) {
    const stringWithFolderId = event.target.value;
    if (!stringWithFolderId) return;
    googleDriveFolderOrFileId = getGDriveFileOrFolderId(stringWithFolderId);
    validateFileIdInput();
  }

  function validateFileIdInput() {
    if (!isValidFolderIdInput) {
      setTimeout(() => {
        googleDriveFolderOrFileId = '';
        isValidFolderIdInput = true;
      }, 3000);
    }
  }

  async function deleteFile(id) {
    const fileIds = queue.filter((value) => value.id !== id).map((x) => x.id);
    await updatePlaylist(fileIds);
  }

  async function updatePlaylist(fileIds) {
    const variables = { restreamId, fileIds };
    try {
      await setPlaylist({ variables });
    } catch (e) {
      showError(e.message);
    }
  }

  async function startStopPlaying(fileId) {
    try {
      if (currentlyPlayingFileId === fileId) {
        const variables = { restreamId };
        await stopPlayingFileFromPlaylist({ variables });
      } else {
        const variables = { restreamId, fileId };
        await playFileFromPlaylist({ variables });
      }
    } catch (e) {
      showError(e.message);
    }
  }

  function handleSort(e) {
    queue = e.detail.items;
    dragDisabled = true;
  }

  async function onDrop(e) {
    handleSort(e);

    const fileIds = queue.map((x) => x.id);
    await updatePlaylist(fileIds);
  }

  function startDrag(e) {
    // preventing default to prevent lag on touch devices (because of the browser checking for screen scrolling)
    e.preventDefault();
    dragDisabled = false;
  }
</script>

<template>
  <div class="playlist">
    <div class="uk-flex uk-flex-middle uk-margin-bottom playlist-toolbar">
      <PlaylistStatus files={queue} />
    </div>
    <div class="google-drive-dir uk-flex uk-flex-middle">
      <label for="gdrive">Add files from Google Drive</label>
      <input
        id="gdrive"
        bind:value={googleDriveFolderOrFileId}
        on:input={handleInputFolderId}
        class="google-drive-link uk-input uk-form-small uk-flex-1"
        type="text"
        disabled={!isValidFolderIdInput}
        placeholder="ID of Google Drive folder"
      />

      <button
        disabled={!googleDriveFolderOrFileId.trim()}
        class="uk-button uk-button-primary uk-button-small uk-flex-none load-file"
        on:click={() => loadPlaylistOrFile(googleDriveFolderOrFileId)}
      >
        <i class="uk-icon" uk-icon="cloud-download" />&nbsp;<span
          >Load files</span
        >
      </button>
      <Confirm let:confirm>
        <button
          class="uk-button uk-button-link url-action-btn start-download"
          class:uk-hidden={!hasFilesInPlaylist || hasDownloadingFiles}
          data-testid="start-all-outputs"
          title="Start all incomplete downloads of files in the playlist"
          on:click={() => confirm(startPlaylistDownload)}
          >Start all downloads
          <i class="uk-icon" uk-icon="icon: cloud-download; ratio: 0.8" />&nbsp;
        </button>
        <span slot="title">Start all downloads</span>
        <span slot="description"
          >This will restart all not complete downloads of files in playlist.
        </span>
        <span slot="confirm">Start downloads</span>
      </Confirm>

      <Confirm let:confirm>
        <button
          class="uk-button uk-button-link url-action-btn stop-download"
          class:uk-hidden={!hasFilesInPlaylist || !hasDownloadingFiles}
          data-testid="stop-all-outputs"
          title="Stop all downloads of all files in the playlist"
          on:click={() => confirm(stopPlaylistDownload)}
          value=""
          >Cancel all downloads <i
            class="uk-icon"
            uk-icon="icon: ban; ratio: 0.8"
          />&nbsp;
        </button>
        <span slot="title">Cancel all active downloads</span>
        <span slot="description"
          >This will stop active downloads of files in playlist.
        </span>
        <span slot="confirm">Stop downloads</span>
      </Confirm>

      <Confirm let:confirm>
        <button
          class="uk-button uk-button-link url-action-btn uk-margin-auto-left clear-playlist"
          class:uk-hidden={!hasFilesInPlaylist || hasDownloadingFiles}
          data-testid="clear-playlist"
          title="Clear playlist"
          on:click={() => confirm(clearPlaylist)}
          value=""
          >Clear playlist
        </button>
        <span slot="title">Clear all files in playlist</span>
        <span slot="description">All files will be removed from playlist.</span>
        <span slot="confirm">Clear playlist</span>
      </Confirm>
    </div>
    <div
      class="playlist-items"
      use:dndzone={{
        items: queue,
        dropTargetClasses: ['drop-target'],
        dragDisabled,
        flipDurationMs: 200,
      }}
      on:consider={handleSort}
      on:finalize={onDrop}
    >
      {#each queue as item (item.id)}
        <div class="item uk-card uk-card-default">
          <span
            class="item-drag-zone uk-icon"
            class:uk-invisible={hasDownloadingFiles}
            uk-icon="table"
            on:mousedown={startDrag}
          />
          <Confirm let:confirm>
            <span slot="title"
              >{item.isPlaying ? 'Stop' : 'Start'} playing file</span
            >
            <span slot="description" />
            <span slot="confirm">{item.isPlaying ? 'Stop' : 'Start'}</span>
            <div
              class="item-file uk-height-1-1 uk-width-1-1 uk-flex uk-flex-middle"
              class:is-playing={item.isPlaying}
              class:is-finished={item.wasPlayed}
            >
              <!-- svelte-ignore a11y-click-events-have-key-events -->
              <span
                class="item-icon"
                class:can-be-started={item.isLocal}
                on:click={() =>
                  item.isLocal
                    ? confirm(() => startStopPlaying(item.id))
                    : undefined}
              >
                {#if !item.isLocal}
                  <span
                    class="file-icon"
                    class:is-downloading={item.isDownloading}
                  >
                    {#if item.isError}
                      <Fa icon={faTriangleExclamation} />
                    {:else}
                      <Fa icon={faDownload} />
                    {/if}
                  </span>
                {:else if item.isPlaying}
                  <Fa icon={faCircleStop} />
                {:else}
                  <Fa icon={faCirclePlay} />
                {/if}
              </span>
              {#if item.file}
                <FileInfo
                  file={item.file}
                  showDownloadLink={true}
                  classList="uk-margin-small-left"
                />
              {/if}
            </div>
          </Confirm>
          <Confirm let:confirm>
            <span slot="title">Delete file from playlist</span>
            <span slot="description"
              >This action will stop playing and delete file from playlist</span
            >
            <span slot="confirm">Delete</span>
            <button
              type="button"
              class="uk-close"
              class:uk-hidden={hasDownloadingFiles}
              uk-close
              on:click={() => confirm(() => deleteFile(item.id))}
            />
          </Confirm>
        </div>
      {:else}
        <div
          class="uk-section uk-section-xsmall uk-text-center uk-padding-remove"
        >
          <div class="no-files uk-text-middle uk-card uk-card-default">
            No files in playlist
          </div>
        </div>
      {/each}
    </div>
  </div>
</template>

<style lang="stylus">
  .google-drive-dir
    input
      max-width: 500px
      margin-right: 8px
      margin-left: 8px

  .playlist
    padding: 16px

    &:hover
      .start-download, .stop-download, .load-file, .clear-playlist
        opacity: 1

  .start-download, .stop-download, .load-file, .clear-playlist
    opacity: 0

  .clear-playlist
    color: var(--danger-color)

  .playlist-toolbar
    gap: 4px

  .playlist-items
    margin-top: 8px

    & > *
      margin-top: 4px;

  .no-files
    line-height: 4em
    background-color: #fff

  .item
    display: flex
    align-items: center
    align-content: left
    min-height: 4em
    background-color: #fff

    &:hover
      backgrouncd-color: #f8f8f8

      .uk-close
        display: block

    .uk-close
      display: none
      padding: 8px

  .item-drag-zone
    padding: 8px
    cursor: grab

  .item-icon
    padding-right: 4px
    padding-left: 4px
    font-size: 28px
    color: #949494

    &.can-be-started
      cursor: pointer

    .file-icon
      :global(svg)
        font-size: 28px
        vertical-align: baseline

    .is-downloading
      color: var(--warning-color)

  .item-file
    flex: 1

    &.is-playing
      .item-icon
        color: var(--success-color)

    &.is-finished
      opacity: 0.4

</style>
