<script lang="js">
  import { copyToClipboard } from '../../utils/util';

  export let url;
  export let previewUrl;
</script>

<template>
  <div class="url">
    <span class="url-placeholder">{url}</span>
    {#if previewUrl}
      <span class="url-preview"
        >&nbsp;[<a href={previewUrl} target="_blank">Preview</a>]</span
      >
    {/if}
    <button
      type="button"
      class="url-copy-btn uk-button uk-button-link uk-button-small uk-margin-small-left"
      on:click|preventDefault={() => copyToClipboard(url)}
    >
      Copy
      <i class="far fa-copy"/>
    </button>
  </div>
</template>

<style lang="stylus">
  .url
    display: inline-flex
  .url-placeholder
    word-break: break-all
  .url-preview
    margin-left: 4px
  .url-copy-btn
    align-self: flex-start
    opacity: 0
    text-transform: initial
    text-decoration: none
    font-size: 13px;
    transition: 0.1s ease-in
  .url:hover .url-copy-btn
    opacity: 1;
    vertical-align: baseline;
</style>
