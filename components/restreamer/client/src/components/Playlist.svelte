<script lang='js'>
  import orderBy from 'lodash/orderBy';
  import Confirm from './common/Confirm.svelte';
  import { dndzone } from 'svelte-dnd-action';

  let dragDisabled = true;
  const flipDurationMs = 200;

  $: playlist = [];

  let remote_playlist = [{
    id: 1,
    name: 'File1',
    isPlaying: false,
    isFinished: true
  },
    {
      id: 2,
      name: 'File2',
      isPlaying: false,
      isFinished: true
    },
    {
      id: 3,
      name: 'Потребительский Формат_Consumer Format',
      isPlaying: true,
      isFinished: false
    },
    {
      id: 4,
      name: 'Melting ice and viruses_Таяние льдов и вирусы',
      isPlaying: false,
      isFinished: false
    },
    {
      id: 5,
      name: 'Alternative Sources of Energy_Альтерантивные источники энергии',
      isPlaying: false,
      isFinished: false
    },
    {
      id: 6,
      name: 'Suzuki and Greta. A Scheme of Great Fraud. СО2_Ролик Грета и Сузуки_История великого обмана СО2',
      isPlaying: false,
      isFinished: false
    }];

  let googleDriveUrl = '';

  function getOrderedPlaylist(list) {
    return orderBy(list, ['isFinished', 'isPlaying', 'name'], ['desc', 'desc', 'asc']);
  }

  function loadPlaylist() {
    playlist = getOrderedPlaylist(remote_playlist);
  }

  const getById = (id) => {
    return playlist.find((value) => value.id === id);
  };

  function deleteFile(id) {
    playlist = playlist.filter((value) => {
      return value.id !== id;
    });
  }

  function startStopPlaying(id) {
    const current = getById(id);
    playlist.forEach(x => {
      x.isPlaying = x.id === id ? !x.isPlaying : false;
      if(x.id === id) {
        x.isFinished = !x.isPlaying;
      }
    })
    ;
    playlist = getOrderedPlaylist(playlist);
  }


  function handleSort(e) {
    playlist = e.detail.items;
    dragDisabled = true;
  }

  function startDrag(e) {
    // preventing default to prevent lag on touch devices (because of the browser checking for screen scrolling)
    e.preventDefault();
    dragDisabled = false;
  }

</script>

<template>
  <div class='playlist'>
    <div class='google-drive-dir uk-flex'>
      <label class='uk-flex-none'>
        Add files from google drive
      </label>
      <input
        bind:value={googleDriveUrl}
        class='google-drive-link uk-input uk-form-small uk-flex-1'
        type='text'
        placeholder='Add link to Google Drive folder'
      />
      <button
        disabled={!googleDriveUrl.trim()}
        class='uk-button uk-button-primary uk-button-small uk-flex-none'
        on:click={() => loadPlaylist()}
      >
        <i class='uk-icon' uk-icon='cloud-download' />&nbsp;<span>Load files</span>
      </button>

    </div>

    <div class='playlist-items' use:dndzone={{items: playlist, dropTargetClasses: ["drop-target"], dragDisabled, flipDurationMs, }} on:consider={handleSort} on:finalize={handleSort} >

        {#each playlist as item(item.id)}
          <div class='item'>
            <span class='item-drag-zone uk-icon' uk-icon='table' tabindex=0 on:mousedown={startDrag} ></span>

            <Confirm let:confirm>
              <span slot="title">{item.isPlaying ? 'Stop' : 'Start'} playing file</span>
              <span slot="description"></span>
              <span slot="confirm">{item.isPlaying ? 'Stop' : 'Start'}</span>
                <div class="item-name uk-height-1-1 uk-width-1-1"
                     class:is-playing={item.isPlaying}
                     class:is-finished={item.isFinished}
                     on:click={() => confirm(() => startStopPlaying(item.id))}
                >
                  <span class='item-icon uk-icon'
                        uk-icon={item.isPlaying ? "icon: future; ratio: 2.5" : "icon: youtube; ratio: 2.5"}
                  ></span>
                  <span>{item.name}</span>
                </div>
            </Confirm>
            <Confirm let:confirm>
              <span slot="title">Delete file</span>
              <span slot="description">This action will stop playing and delete file from playlist</span>
              <span slot="confirm">Delete</span>
                <button
                  type='button'
                  class='uk-close'
                  uk-close
                  on:click={() => confirm(() => deleteFile(item.id))}
                />
            </Confirm>
          </div>
        {:else}
          <div class='uk-section uk-section-xsmall uk-text-center'>
            <div>
              No files in playlist
            </div>
          </div>
        {/each}
    </div>
  </div>
</template>

<style lang='stylus'>
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
      border: 1px solid #ddd

  .item
    display: flex
    align-items: center
    align-content: left
    min-height: 4em

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
