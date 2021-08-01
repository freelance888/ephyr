<script lang="js">
  import { onMount } from 'svelte';
  import { getYoutubeVideoID } from '../util';

  export let restream_id;
  export let preview_url;

  let iframeVideoURL = '';

  onMount(() => {
    iframeVideoURL = updateVideoURL();
  });

  function updateVideoURL() {
    let videoID = getYoutubeVideoID(preview_url);
    return `https://www.youtube.com/embed/${videoID}?autoplay=1&controls=1`;
  }
</script>

<template>
  <div class="wise-iframe-wrapper">
    <iframe
      src={iframeVideoURL}
      title="YouTube video player"
      frameborder="0"
      allowfullscreen
    />
  </div>
</template>

<style lang="stylus">
    .wise-iframe-wrapper
        position: relative
        padding-bottom: 56.10%
        height: 0
        overflow: hidden

    .wise-iframe-wrapper iframe,
    .wise-iframe-wrapper object,
    .wise-iframe-wrapper embed
        position: absolute;
        top: 0;
        left: 0;
        width: 100%;
        height: auto;
</style>
