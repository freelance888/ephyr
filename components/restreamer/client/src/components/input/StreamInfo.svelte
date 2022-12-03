<script lang="js">
  export let streamInfo;
</script>

<template>
  <div class='stream-info'>
    <div>
      <span><strong>video:</strong> {streamInfo.videoCodecName}, </span>
      <span>{streamInfo.videoWidth} * {streamInfo.videoHeight},</span>
      <span>{streamInfo.videoRFrameRate} FPS</span>
    </div>

    <div>
      <span><strong>audio:</strong> {streamInfo.audioCodecName},</span>
      <span>{streamInfo.audioSampleRate},</span>
      <span>{streamInfo.audioChannelLayout},</span>
      <span>channels: {streamInfo.audioChannels}</span>
    </div>
  </div>
</template>

<style lang="stylus">
  .stream-info
    padding: 4px
    color: #1e87f0
    font-size: smaller
</style>
