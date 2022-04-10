<script lang="js">

  let playlist1 = [];
  let playlist = [{
    name: "File1",
    isPlaying: false,
    isFinished: true
  },
  {
    name: "File2",
    isPlaying: false,
    isFinished: true
  },
  {
    name: "Потребительский Формат_Consumer Format",
    isPlaying: true,
    isFinished: false
  },
  {
    name: "Melting ice and viruses_Таяние льдов и вирусы",
    isPlaying: false,
    isFinished: false
  },
  {
    name: "Alternative Sources of Energy_Альтерантивные источники энергии",
    isPlaying: false,
    isFinished: false
  },
  {
    name: "Suzuki and Greta. A Scheme of Great Fraud. СО2_Ролик Грета и Сузуки_История великого обмана СО2",
    isPlaying: false,
    isFinished: false
  }];
</script>

<template>
  <div class="playlist">
    <div class="google-drive-dir uk-flex">
      <label class="uk-flex-none">
        Add files from google drive
      </label>
      <input
        class="google-drive-link uk-input uk-form-small uk-flex-1"
        type="text"
        placeholder="Add link to Google Drive folder"
      />
      <button
        class="uk-button uk-button-primary uk-button-small uk-flex-none"
        on:click={() => {}}
      >
        <i class="fas fa-plus" />&nbsp;<span>Load files</span>
      </button>

    </div>
    <div class="playlist-items">
      {#each playlist as item}
        <div class="item">
          <span class="item-drag-zone uk-icon" uk-icon="table"></span>
          <span class="item-icon uk-icon"
                class:is-playing={item.isPlaying}
                class:is-finished={item.isFinished}
                uk-icon="youtube"
          ></span>
          <span class="item-name"
                class:is-playing={item.isPlaying}
          >{item.name}</span>
          <button
            type="button"
            class="uk-close"
            uk-close
            on:click={() => {}}
          />

        </div>
      {:else}
        <div class="uk-section uk-section-xsmall uk-text-center">
          <div>
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

  .playlist-items
    margin-top: 8px
    & > *
      border-top: 1px solid #ddd
    & > *:last-child
      border-bottom: 1px solid #ddd

  .item
    display: flex
    align-items: center
    align-content: left
    min-height: 4em
    .item-icon
      padding-right: 4px
      padding-left: 4px
      &.is-playing :global(svg)
        height: 80px
      &.is-finished
        opacity: 0.5
      :global(svg)
        height: 50px
        width: auto
    .item-name
      flex: 1
      padding: 8px
      &.is-playing
        font-weight: 700

</style>
