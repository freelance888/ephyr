<script lang="js">
  import Fa from 'svelte-fa';
  import { faCopy } from '@fortawesome/free-regular-svg-icons';

  import { copyToClipboard } from '../../utils/util';
  import StreamInfo from './StreamInfo.svelte';

  export let url;
  export let previewUrl = false;
  export let streamInfo;
  export let isError;
</script>

<template>
  <div class="url">
    {#if url}
      <span class="url-placeholder">{url}</span>
    {/if}
    {#if streamInfo}
      <StreamInfo {streamInfo} {isError} />
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
        class="url-action-btn uk-button uk-button-link"
        on:click|preventDefault={() => copyToClipboard(url)}
      >
        Copy
        <Fa icon={faCopy} />
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
      .url-action-btn
        opacity: 1
        vertical-align: baseline

  .url-placeholder
    word-break: break-all

  .url-preview
    margin-left: 4px

</style>
