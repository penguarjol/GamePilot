import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { toast } from "sonner";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui/tabs";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import type { PlayerStats } from "@/types";

interface GEPriceResult {
  item_id: number;
  name: string;
  price: number;
  high: number;
  low: number;
}

export function RuneScape() {
  const [username, setUsername] = useState("");
  const [game, setGame] = useState<"osrs" | "rs3">("osrs");
  const [stats, setStats] = useState<PlayerStats | null>(null);
  const [lookingUp, setLookingUp] = useState(false);

  const [itemId, setItemId] = useState("");
  const [priceResult, setPriceResult] = useState<GEPriceResult | null>(null);
  const [priceLooking, setPriceLooking] = useState(false);

  const lookupPlayer = async () => {
    if (!username.trim()) return;
    setLookingUp(true);
    setStats(null);
    try {
      const result = await invoke<PlayerStats>("lookup_runescape_player", {
        username: username.trim(),
        game,
      });
      setStats(result);
    } catch (err) {
      toast.error(`Lookup failed: ${err}`);
    } finally {
      setLookingUp(false);
    }
  };

  const lookupPrice = async () => {
    const id = parseInt(itemId.trim(), 10);
    if (isNaN(id)) {
      toast.error("Enter a valid item ID");
      return;
    }
    setPriceLooking(true);
    setPriceResult(null);
    try {
      const result = await invoke<GEPriceResult>("lookup_ge_price", {
        itemId: id,
      });
      setPriceResult(result);
    } catch (err) {
      toast.error(`Price lookup failed: ${err}`);
    } finally {
      setPriceLooking(false);
    }
  };

  const formatXP = (xp: number) => xp.toLocaleString();
  const formatRank = (rank: number) =>
    rank > 0 ? `#${rank.toLocaleString()}` : "Unranked";

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold tracking-tight">RuneScape</h1>
        <p className="text-sm text-muted-foreground mt-1">
          Player stats lookup and Grand Exchange prices
        </p>
      </div>

      {/* Player Lookup */}
      <Card>
        <CardHeader>
          <CardTitle className="text-base">Player Lookup</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <Tabs
            value={game}
            onValueChange={(v) => setGame(v as "osrs" | "rs3")}
          >
            <TabsList>
              <TabsTrigger value="osrs">OSRS</TabsTrigger>
              <TabsTrigger value="rs3">RS3</TabsTrigger>
            </TabsList>
            <TabsContent value="osrs" />
            <TabsContent value="rs3" />
          </Tabs>

          <div className="flex gap-2">
            <input
              type="text"
              value={username}
              onChange={(e) => setUsername(e.target.value)}
              onKeyDown={(e) => e.key === "Enter" && lookupPlayer()}
              placeholder="Enter username..."
              className="flex-1 h-8 rounded-lg border border-border bg-background px-3 text-sm placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring"
            />
            <Button
              onClick={lookupPlayer}
              disabled={lookingUp || !username.trim()}
            >
              {lookingUp ? "Looking up..." : "Lookup"}
            </Button>
          </div>
        </CardContent>
      </Card>

      {/* Player Stats */}
      {stats && (
        <>
          <Card>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-base">{stats.username}</CardTitle>
              <Badge variant="outline">
                {game === "osrs" ? "Old School" : "RS3"}
              </Badge>
            </CardHeader>
            <CardContent>
              <div className="flex gap-6 text-sm">
                <div>
                  <span className="text-muted-foreground">Total Level</span>
                  <p className="text-2xl font-bold">
                    {stats.total_level.toLocaleString()}
                  </p>
                </div>
                <div>
                  <span className="text-muted-foreground">Total XP</span>
                  <p className="text-2xl font-bold">{formatXP(stats.total_xp)}</p>
                </div>
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle className="text-base">
                Skills ({stats.skills.length})
              </CardTitle>
            </CardHeader>
            <CardContent>
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>Skill</TableHead>
                    <TableHead className="text-right">Level</TableHead>
                    <TableHead className="text-right">XP</TableHead>
                    <TableHead className="text-right">Rank</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {stats.skills.map((skill) => (
                    <TableRow key={skill.name}>
                      <TableCell className="font-medium">
                        {skill.name}
                      </TableCell>
                      <TableCell className="text-right font-mono">
                        {skill.level}
                      </TableCell>
                      <TableCell className="text-right font-mono">
                        {formatXP(skill.xp)}
                      </TableCell>
                      <TableCell className="text-right text-muted-foreground">
                        {formatRank(skill.rank)}
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </CardContent>
          </Card>
        </>
      )}

      {/* GE Price Lookup */}
      <Card>
        <CardHeader>
          <CardTitle className="text-base">Grand Exchange</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex gap-2">
            <input
              type="text"
              value={itemId}
              onChange={(e) => setItemId(e.target.value)}
              onKeyDown={(e) => e.key === "Enter" && lookupPrice()}
              placeholder="Item ID..."
              className="w-48 h-8 rounded-lg border border-border bg-background px-3 text-sm placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring"
            />
            <Button
              onClick={lookupPrice}
              disabled={priceLooking || !itemId.trim()}
            >
              {priceLooking ? "Searching..." : "Search"}
            </Button>
          </div>

          {priceResult && (
            <div className="rounded-lg border border-border p-4">
              <h4 className="font-medium text-sm mb-2">{priceResult.name}</h4>
              <dl className="grid grid-cols-[auto_1fr] gap-x-3 gap-y-1 text-sm">
                <dt className="text-muted-foreground">Price</dt>
                <dd className="font-mono">
                  {priceResult.price.toLocaleString()} gp
                </dd>
                <dt className="text-muted-foreground">High</dt>
                <dd className="font-mono">
                  {priceResult.high.toLocaleString()} gp
                </dd>
                <dt className="text-muted-foreground">Low</dt>
                <dd className="font-mono">
                  {priceResult.low.toLocaleString()} gp
                </dd>
              </dl>
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
