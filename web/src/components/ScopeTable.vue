<script setup lang="ts">
import { Badge } from '@/components/ui/badge'
import {
  Table,
  TableBody,
  TableCell,
  TableEmpty,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { age } from '@/lib/age'
import type { Scope } from '@/lib/scopes'

const props = defineProps<{
  scopes: Scope[]
  /** The URL segment the scope's page lives under: `sessions` or `changes`. */
  prefix: string
}>()
</script>

<template>
  <Table>
    <TableHeader>
      <TableRow>
        <!-- The flexible column. Every cell is `whitespace-nowrap` by default, so
             without this the names and ids push the table past the card and the
             verdict on the far right is cut off. -->
        <TableHead class="w-full">Name</TableHead>
        <TableHead>Modified</TableHead>
        <TableHead>Open comments</TableHead>
        <TableHead>Verdict</TableHead>
      </TableRow>
    </TableHeader>
    <TableBody>
      <TableEmpty v-if="props.scopes.length === 0" :colspan="4">
        None discovered.
      </TableEmpty>
      <TableRow v-for="scope in props.scopes" :key="scope.key">
        <TableCell class="whitespace-normal">
          <div class="flex flex-wrap items-center gap-2">
            <!-- Addressed by identifier and never by title: the title is read
                 from a file the agent rewrites, so a URL built from it would
                 break the moment the exploration's topic shifted. -->
            <a
              class="font-medium underline-offset-4 hover:underline"
              :href="`/${props.prefix}/${scope.key}`"
            >{{ scope.title ?? scope.key }}</a>
            <!-- Deliberately not "live": nothing here says the session is still
                 running, only that it is the one that spoke to the reviewer
                 last. -->
            <Badge v-if="scope.mostRecentlyActive" variant="outline">
              most recently active
            </Badge>
          </div>
          <!-- The identifier is on the row whether or not it leads with a
               title, because it is what an operator pastes into
               `openspec-doc comment list`. -->
          <code class="text-xs text-muted-foreground">{{ scope.key }}</code>
        </TableCell>
        <TableCell class="text-muted-foreground">
          {{ scope.modifiedAt ? age(scope.modifiedAt) : '—' }}
        </TableCell>
        <TableCell>
          <Badge v-if="scope.openComments > 0" variant="default">
            {{ scope.openComments }}
          </Badge>
          <span v-else class="text-muted-foreground">0</span>
        </TableCell>
        <TableCell>
          <Badge v-if="scope.verdict" class="bg-verdict text-verdict-foreground">{{ scope.verdict }}</Badge>
          <span v-else class="text-muted-foreground">—</span>
        </TableCell>
      </TableRow>
    </TableBody>
  </Table>
</template>
