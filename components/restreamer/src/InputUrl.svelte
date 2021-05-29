<script lang="js">
  import { copyToClipboard } from './util';

  export let public_host = '167.172.35.18';
  export let endpoint;
  export let restream_key;
  export let value;
  export let is_pull;

  let inputUrl = () => {
    if (endpoint.kind === 'HLS')
      return `http://${public_host}:8000/hls/${restream_key}/${value.key}.m3u8`;
    else if (is_pull) return value.src.url;
    else return `rtmp://${public_host}/${restream_key}/${value.key}`;
  };
</script>

<template>
  <div class="input-url">
    <span>{inputUrl()}</span>
    <button
      type="button"
      class="input-url-copy-btn uk-button uk-button-link uk-button-small uk-margin-small-left"
      on:click|preventDefault={() => copyToClipboard(inputUrl())}
    >
      Copy
    </button>
  </div>
</template>

<style lang="stylus">
  .input-url
    display: inline-flex
  .input-url-copy-btn
    opacity: 0
    text-transform: initial
    text-decoration: none
    transition: 0.1s ease-in
  .input-url:hover .input-url-copy-btn
    opacity: 1;
    vertical-align: baseline;
</style>
