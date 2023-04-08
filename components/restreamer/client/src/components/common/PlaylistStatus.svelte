<script lang="js">
  export let files;

  let filesState;
  let done = false;

  $: {
    filesState = {
      DOWNLOADING: 0,
      WAITING: 0,
      LOCAL: 0,
      DOWNLOAD_ERROR: 0,
    };

    files.forEach(
      ({ file }) => (filesState[file?.state] = filesState[file?.state] + 1)
    );

    done = filesState?.LOCAL === files.length && files.length > 0;
  }
</script>

<template>
  <div
    class="uk-flex"
    class:bg-ready={done}
    class:bg-error={filesState?.DOWNLOAD_ERROR}
  >
    {#if done}
      <div class="status-item status-ready">Playlist is ready</div>
    {:else}
      <div class="status-item">pending: {filesState?.WAITING}</div>
      <hr class="uk-divider-vertical" />
      <div class="status-item">downloading: {filesState?.DOWNLOADING}</div>
      <hr class="uk-divider-vertical" />
      <div class="status-item">local: {filesState?.LOCAL}</div>
      {#if filesState?.DOWNLOAD_ERROR > 0}
        <hr class="uk-divider-vertical" />
        <div class="status-item">error: {filesState?.DOWNLOAD_ERROR}</div>
      {/if}
    {/if}
  </div>
</template>

<style lang="stylus">

    .uk-flex
        margin-left: 10px
        align-items: center
        background: #eee
        max-height: 2em
        border-radius: 0.4em

    .status-item
        font-size: 14px
        padding: 0 20px 

    .status-ready 
        font-size: 16px
    
    .bg-ready
        background: #ded
    
    .bg-error 
        background: #edd

    .uk-divider-vertical
        height: 25px

</style>
