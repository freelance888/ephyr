<script lang="js">
  import Confirm from './common/Confirm.svelte';
  import { dndzone } from 'svelte-dnd-action';

  import {
    GetPlaylistFromGdrive,
    PlayFileFromPlaylist,
    SetPlaylist,
    StopPlayingFileFromPlaylist
  } from '../../api/client.graphql';
  import { mutation } from 'svelte-apollo';
  import { showError } from '../utils/util';

  const getPlaylistFromDrive = mutation(GetPlaylistFromGdrive);
  const setPlaylist = mutation(SetPlaylist);
  const playFileFromPlaylist = mutation(PlayFileFromPlaylist);
  const stopPlayingFileFromPlaylist = mutation(StopPlayingFileFromPlaylist);

  let dragDisabled = true;
  const flipDurationMs = 200;

  export let restreamId;
  export let playlist;

  $: queue = playlist
    ? playlist.queue.map((x) => ({
        id: x.fileId,
        name: x.name,
        isPlaying: playlist.currentlyPlayingFile
          ? playlist.currentlyPlayingFile.fileId === x.fileId
          : false,
        wasPlayed: x.wasPlayed,
      }))
    : [];
  $: hasPlaylistLoaded = queue && queue.length > 0;
  let googleDriveFolderId = '';

  async function loadPlaylist(folderId) {
    const variables = { id: restreamId, folder_id: folderId };
    try {
      await getPlaylistFromDrive({ variables });
      googleDriveFolderId = '';
    } catch (e) {
      showError(e.message);
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

  async function startStopPlaying(file_id) {
    try {
      if (
        playlist.currentlyPlayingFile &&
        playlist.currentlyPlayingFile.fileId === file_id
      ) {
        const variables = { restreamId };
        await stopPlayingFileFromPlaylist({ variables });
      } else {
        const variables = { restreamId, file_id };
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
    <div class="google-drive-dir uk-flex">
      <label class="uk-flex-none">
        <input
          bind:value={googleDriveFolderId}
          class="google-drive-link uk-input uk-form-small uk-flex-1"
          type="text"
          placeholder="Add link to Google Drive folder"
        /> Add files from Google Drive
      </label>
      <button
        disabled={!googleDriveFolderId.trim()}
        class="uk-button uk-button-primary uk-button-small uk-flex-none"
        on:click={() => loadPlaylist(googleDriveFolderId)}
      >
        <i class="uk-icon" uk-icon="cloud-download" />&nbsp;<span
          >Load files</span
        >
      </button>
    </div>
    <div
      class="playlist-items"
      use:dndzone={{
        items: queue,
        dropTargetClasses: ['drop-target'],
        dragDisabled,
        flipDurationMs,
      }}
      on:consider={handleSort}
      on:finalize={onDrop}
    >
      {#each queue as item (item.id)}
        <div class="item uk-card uk-card-default">
          <span
            class="item-drag-zone uk-icon"
            uk-icon="table"
            tabindex="0"
            on:mousedown={startDrag}
          />

          <Confirm let:confirm>
            <span slot="title"
              >{item.isPlaying ? 'Stop' : 'Start'} playing file</span
            >
            <span slot="description" />
            <span slot="confirm">{item.isPlaying ? 'Stop' : 'Start'}</span>
            <div
              class="item-name uk-height-1-1 uk-width-1-1"
              class:is-playing={item.isPlaying}
              class:is-finished={item.wasPlayed}
              on:click={() => confirm(() => startStopPlaying(item.id))}
            >
              <span
                class="item-icon uk-icon"
                uk-icon={item.isPlaying
                  ? 'icon: future; ratio: 2.5'
                  : 'icon: youtube; ratio: 2.5'}
              />
              <span>{item.name}</span>
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
  :global(.drop-target) {
    outline: none !important;
  }

  .google-drive-dir
    input
      max-width: 500px
      margin-right: 8px
      margin-left: 8px

  .playlist
    padding: 16px

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
      background-color: #f8f8f8
      cursor: pointer

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

  .item-name
    flex: 1
    padding: 8px
    &.is-playing
      font-weight: 700

    &.is-finished
      opacity: 0.4

</style>