<script lang="js">
  import { copyToClipboard } from '../../utils/util';

  export let url;
  export let previewUrl;
  export let hasStreamInfo;
</script>

<template>
  <div class="url">
    <span class="url-placeholder">{url}</span>
    {#if hasStreamInfo}
      <span uk-icon="icon: info; ratio: 0.7"></span>
    {/if}
    {#if previewUrl}
      <span class="url-preview"
        >&nbsp;[<a href={previewUrl} target="_blank">Preview</a>]</span
      >
    {/if}
    <button
      class="url-copy-btn uk-button uk-button-link uk-margin-small-left"
      on:click|preventDefault={() => copyToClipboard(url)}
    >
      Copy
      <i class="far fa-copy" />
    </button>
  </div>
</template>

<style lang="stylus">
  .url
    display: inline-flex
    min-width: 20em

    &:hover
      .url-copy-btn
        opacity: 1
        vertical-align: baseline


  .url-placeholder
    word-break: break-all

  .url-preview
    margin-left: 4px

  .url-copy-btn
    align-self: center
    color: var(--primary-text-color)
    opacity: 0
    text-transform: initial
    text-decoration: none
    font-size: 13px
    transition: 0.1s ease-in
    &:hover
      color: var(--primary-text-hover-color)
</style>
