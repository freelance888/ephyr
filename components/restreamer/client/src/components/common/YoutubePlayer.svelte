<script lang="js">
  import { onMount } from 'svelte';
  import { getYoutubeVideoID } from '../../utils/util';

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

        iframe
          position: absolute;
          top: 0;
          left: 0;
          width: 100%;
          height: 100%;
</style>
