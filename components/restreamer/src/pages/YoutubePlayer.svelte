<script lang="js">
  import {onMount} from "svelte";

  export let restream_id;
  export let preview_url;

  let iframeVideoURL = "";

  onMount(() => {
      iframeVideoURL = updateVideoURL();
  })

  function updateVideoURL(){
      const VID_REGEX = /(?:youtube(?:-nocookie)?\.com\/(?:[^\/\n\s]+\/\S+\/|(?:v|e(?:mbed)?)\/|\S*?[?&]v=)|youtu\.be\/)([a-zA-Z0-9_-]{11})/;
      let videoID = preview_url.match(VID_REGEX)[1];
      return `https://www.youtube.com/embed/${videoID}`;
  }
</script>

<template>
  <div class="wise-iframe-wrapper">
  <iframe width="100%" height="auto" src="{iframeVideoURL}"
          title="YouTube video player" frameborder="0"
          allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture"
          allowfullscreen>
  </iframe></div>
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
        height: 100%;
</style>