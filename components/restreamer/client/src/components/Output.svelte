<script lang="js">
  import Fa from 'svelte-fa';
  import { faEdit } from '@fortawesome/free-regular-svg-icons';
  import { faCircle } from '@fortawesome/free-solid-svg-icons';
  import { faDotCircle } from '@fortawesome/free-regular-svg-icons';
  import { faDotCircle as faDotCircleSolid } from '@fortawesome/free-solid-svg-icons';
  import { faExternalLinkAlt } from '@fortawesome/free-solid-svg-icons';

  import { mutation } from 'svelte-apollo';
  import { getMixPageUrl, showError } from '../utils/util';

  import { outputModal } from '../stores';

  import Confirm from './common/Confirm.svelte';
  import Toggle from './common/Toggle.svelte';
  import Volume from './common/Volume.svelte';
  import Mixin from './Mixin.svelte';
  import RecordsModal from '../modals/RecordsModal.svelte';
  import Url from './common/Url.svelte';
  import { createEventDispatcher } from 'svelte';

  export let public_host;
  export let value;
  export let restream_id;
  export let hidden = false;
  export let deleteConfirmation;
  export let enableConfirmation;
  export let mutations;
  export let isReadOnly = false;
  export let outputsSortMode = false;

  const dispatch = createEventDispatcher();

  const disableOutputMutation = mutations.DisableOutput
    ? mutation(mutations.DisableOutput)
    : undefined;
  const enableOutputMutation = mutations.EnableOutput
    ? mutation(mutations.EnableOutput)
    : undefined;
  const removeOutputMutation = mutations.RemoveOutput
    ? mutation(mutations.RemoveOutput)
    : undefined;

  $: toggleStatusText = value.enabled ? 'Disable' : 'Enable';
  $: activeSidechainId = value.mixins.find((m) => m.sidechain === true)?.id;

  async function toggle() {
    const variables = { restream_id, output_id: value.id };
    try {
      if (value.enabled) {
        await disableOutputMutation({ variables });
      } else {
        await enableOutputMutation({ variables });
      }
    } catch (e) {
      showError(e.message);
    }
  }

  async function remove() {
    const variables = { restream_id, output_id: value.id };
    try {
      await removeOutputMutation({ variables });
    } catch (e) {
      showError(e.message);
    }
  }

  function openEditOutputModal() {
    outputModal.openEdit(
      restream_id,
      value.id,
      value.label,
      value.previewUrl,
      value.dst,
      value.mixins.map((m) => m.src)
    );
  }

  function outputStartDrag(e) {
    e.preventDefault();
    dispatch('outputDragStarted', false);
  }
</script>

<template>
  <div
    class="uk-card uk-card-default uk-card-body uk-flex"
    data-testid={value.label}
    class:hidden
    class:grouped={!isReadOnly}
    class:uk-margin-left={!isReadOnly}
  >
    {#if !isReadOnly}
      <Confirm let:confirm>
        <button
          type="button"
          class="uk-close"
          uk-close
          on:click={deleteConfirmation ? () => confirm(remove) : remove}
        />
        <span slot="title">Removing output</span>
        <span slot="description"
          ><code class="overflow-wrap">{value.dst}</code>
          <br /><br />
          {#if value.dst.startsWith('file:///')}
            <b>Warning!</b> Any associated recorded files will be removed.
            <br /><br />
          {/if}
          You won't be able to undone this.</span
        >
        <span slot="confirm">Remove</span>
      </Confirm>
    {/if}

    {#if value.label}
      <span class="label" title={value.label}>{value.label}</span>
    {/if}

    {#if !isReadOnly}
      <div class="left-buttons-area" />
      <span
        class:uk-hidden={!outputsSortMode}
        class="item-drag-zone uk-icon"
        uk-icon="table"
        on:mousedown={outputStartDrag}
      />
      <a
        class:uk-hidden={outputsSortMode}
        class="edit-output"
        href="/"
        title="Edit output"
        on:click|preventDefault={openEditOutputModal}
      >
        <Fa icon={faEdit} />
      </a>

      <div>
        <Confirm let:confirm>
          <Toggle
            id="output-toggle-{value.id}"
            data-testid="toggle-output-status"
            classes="small"
            checked={value.enabled}
            confirmFn={enableConfirmation ? confirm : undefined}
            onChangeFn={toggle}
          />
          <span slot="title" class="output-name"
            >{toggleStatusText} <code>{value.dst}</code> output</span
          >
          <span slot="description">Are you sure about it?</span>
          <span slot="confirm">{toggleStatusText}</span>
        </Confirm>
      </div>
    {/if}

    <div class="output-mixes">
      <div class="uk-flex uk-flex-base-line">
        {#if value.status === 'ONLINE'}
          <span
            class="uk-margin-small-right status-indicator e-circle online"
            data-testid={`output-status:${value.status}`}
          >
            <Fa icon={faCircle} />
          </span>
        {:else if value.status === 'INITIALIZING'}
          <span
            class="uk-margin-small-right status-indicator e-dot-circle initializing"
            data-testid={`output-status:${value.status}`}
          >
            <Fa icon={faDotCircleSolid} />
          </span>
        {:else if value.status === 'UNSTABLE'}
          <span
            class="uk-margin-small-right status-indicator e-dot-circle unstable"
            data-testid={`output-status:${value.status}`}
          >
            <Fa icon={faDotCircleSolid} />
          </span>
        {:else}
          <span
            class="uk-margin-small-right status-indicator e-dot-circle offline"
            data-testid={`output-status:${value.status}`}
          >
            <Fa icon={faDotCircle} />
          </span>
        {/if}
        {#if value.dst.startsWith('file:///') && value.status === 'OFFLINE'}
          <RecordsModal let:open id={value.id} {public_host}>
            <a
              class="dvr-link"
              href="/"
              on:click|preventDefault={open}
              title="Download records">{value.dst}</a
            >
          </RecordsModal>
        {:else}
          <Url url={value.dst} previewUrl={value.previewUrl} />
        {/if}
      </div>

      {#if value.mixins.length > 0}
        {#if !isReadOnly}
          <a
            class="single-view"
            href={getMixPageUrl(restream_id, value.id)}
            target="_blank"
            rel="noopener noreferrer"
            title="Open in a separate window"
            ><Fa icon={faExternalLinkAlt} />
          </a>
        {/if}

        <Volume
          volume={value.volume}
          {restream_id}
          output_id={value.id}
          {mutations}
        />
        {#each value.mixins as mixin}
          <Mixin
            {restream_id}
            output_id={value.id}
            value={mixin}
            {mutations}
            {activeSidechainId}
          />
        {/each}
      {/if}
    </div>
  </div>
</template>

<style lang="stylus">
  .uk-card
    position: relative
    padding: 6px
    margin-top: 15px !important
    min-width 250px
    font-size: 13px
    &.grouped
      width: calc((100% - (20px * 2)) / 2)
      @media screen and (max-width: 700px)
        width: 100%

    &.hidden
      display: none

    .uk-close
      position: absolute;
      right: 6px;
      top: 9px;

    .label
      position: absolute
      top: -12px
      left: 0
      padding: 0 6px
      font-size: 13px
      border-top-left-radius: 4px
      border-top-right-radius: 4px
      background-color: #fff
      max-width: 85%
      max-height: 19px
      white-space: nowrap
      overflow: hidden
      text-overflow: ellipsis

    a.single-view, a.edit-output
      position: absolute
      outline: none
      transition: opacity .3s ease
      &:hover
        text-decoration: none
    a.single-view
      top: 47px
      left: 16px
      color: #d9d9d9
      &:hover
        color: #c4c4c4
    a.edit-output
      left: -16px
      top: 6px
      color: var(--primary-text-color)
      &:hover
        color: #444
        opacity: 1
    &:not(:hover)
      a.single-view, a.edit-output
        opacity: 0

    .left-buttons-area
      position: absolute
      width: 18px
      right: 100%
      top: 0
      height: 100%

  .status-indicator
    flex-shrink: 0

  .e-circle, .e-dot-circle
    font-size: 10px

  a.dvr-link
    color: var(--primary-text-color)

  .output-mixes
    width: calc(100% - 56px);
    margin-left: 4px

  .item-drag-zone
    margin-right: 4px
    cursor: grab

  .output-name
    overflow: hidden
    text-overflow: ellipsis
    display: inline-block
    width: 100%

</style>
