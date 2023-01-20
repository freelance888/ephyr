<script lang="js">
  import { copyToClipboard } from '../../utils/util';

  export let url;
  export let previewUrl;
  export let streamInfo;
  export let isError;
</script>

<template>
  <div class="url">
    {#if url}
      <span class="url-placeholder">{url}</span>
    {/if}
    {#if streamInfo}
      <span
        class="info-icon"
        class:has-error={isError}
        uk-icon="icon: info; ratio: 0.7"
        uk-tooltip={streamInfo}
      />
    {/if}
    {#if previewUrl}
      <span class="url-preview"
        >&nbsp;[<a href={previewUrl} target="_blank" rel="noopener noreferrer"
          >Preview</a
        >]</span
      >
    {/if}
    {#if url}
      <button
        class="url-copy-btn uk-button uk-button-link uk-margin-small-left"
        on:click|preventDefault={() => copyToClipboard(url)}
      >
        Copy
        <i class="far fa-copy" />
      </button>
    {/if}
  </div>
</template>

<style lang="stylus">
  .url
    align-items: center
    display: inline-flex
    min-width: 20em

    &:hover
      .url-copy-btn
        opacity: 1
        vertical-align: baseline

  .info-icon
    margin-left: 4px

  .url-placeholder
    word-break: break-all

  .url-preview
    margin-left: 4px

  .url-copy-btn
    align-self: center
    height: 100%
    color: var(--primary-text-color)
    opacity: 0
    text-transform: initial
    text-decoration: none
    font-size: 13px
    transition: 0.1s ease-in
    &:hover
      color: var(--primary-text-hover-color)

  .has-error
    color:  var(--danger-color)

</style>
