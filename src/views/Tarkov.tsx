import { useEffect, useState, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import { toast } from "sonner";
import { useInvoke } from "@/hooks/useInvoke";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import type { AmmoData, ItemPrice } from "@/types";

type AmmoSortKey = "name" | "damage" | "penetration" | "armor_damage";
type SortDir = "asc" | "desc";

export function Tarkov() {
  const ammo = useInvoke<AmmoData[]>("get_tarkov_ammo_data");

  const [ammoSortKey, setAmmoSortKey] = useState<AmmoSortKey>("penetration");
  const [ammoSortDir, setAmmoSortDir] = useState<SortDir>("desc");
  const [caliberFilter, setCaliberFilter] = useState<string>("all");

  const [itemQuery, setItemQuery] = useState("");
  const [itemResults, setItemResults] = useState<ItemPrice[]>([]);
  const [itemSearching, setItemSearching] = useState(false);

  useEffect(() => {
    ammo.execute();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const calibers = useMemo(() => {
    if (!ammo.data) return [];
    const set = new Set(ammo.data.map((a) => a.caliber));
    return Array.from(set).sort();
  }, [ammo.data]);

  const filteredAmmo = useMemo(() => {
    if (!ammo.data) return [];
    const filtered =
      caliberFilter === "all"
        ? ammo.data
        : ammo.data.filter((a) => a.caliber === caliberFilter);

    return [...filtered].sort((a, b) => {
      const mul = ammoSortDir === "asc" ? 1 : -1;
      if (ammoSortKey === "name") return mul * a.name.localeCompare(b.name);
      return mul * (a[ammoSortKey] - b[ammoSortKey]);
    });
  }, [ammo.data, caliberFilter, ammoSortKey, ammoSortDir]);

  const toggleAmmoSort = (key: AmmoSortKey) => {
    if (ammoSortKey === key) {
      setAmmoSortDir(ammoSortDir === "asc" ? "desc" : "asc");
    } else {
      setAmmoSortKey(key);
      setAmmoSortDir("desc");
    }
  };

  const ammoSortIndicator = (key: AmmoSortKey) => {
    if (ammoSortKey !== key) return "";
    return ammoSortDir === "asc" ? " \u2191" : " \u2193";
  };

  const searchItems = async () => {
    if (!itemQuery.trim()) return;
    setItemSearching(true);
    try {
      const results = await invoke<ItemPrice[]>("search_tarkov_item", {
        name: itemQuery.trim(),
      });
      setItemResults(results);
    } catch (err) {
      toast.error(`Item search failed: ${err}`);
    } finally {
      setItemSearching(false);
    }
  };

  const formatPrice = (price: number) => {
    if (price >= 1_000_000) return `${(price / 1_000_000).toFixed(1)}M`;
    if (price >= 1_000) return `${(price / 1_000).toFixed(1)}K`;
    return price.toLocaleString();
  };

  const penColor = (pen: number) =>
    pen >= 50
      ? "text-emerald-400"
      : pen >= 30
        ? "text-yellow-400"
        : "text-red-400";

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold tracking-tight">
            Escape from Tarkov
          </h1>
          <p className="text-sm text-muted-foreground mt-1">
            Ammo ballistics and flea market prices
          </p>
        </div>
        <Button
          variant="outline"
          onClick={() => ammo.execute()}
          disabled={ammo.loading}
        >
          {ammo.loading ? "Loading..." : "Refresh"}
        </Button>
      </div>

      {/* Ammo Table */}
      <Card>
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
          <CardTitle className="text-base">Ammo Comparison</CardTitle>
          <Badge variant="outline">{filteredAmmo.length} rounds</Badge>
        </CardHeader>
        <CardContent className="space-y-4">
          {/* Caliber Filter */}
          <div className="flex items-center gap-2 flex-wrap">
            <span className="text-sm text-muted-foreground">Caliber:</span>
            <select
              value={caliberFilter}
              onChange={(e) => setCaliberFilter(e.target.value)}
              className="h-8 rounded-lg border border-border bg-background px-3 text-sm focus:outline-none focus:ring-2 focus:ring-ring"
            >
              <option value="all">All Calibers</option>
              {calibers.map((c) => (
                <option key={c} value={c}>
                  {c}
                </option>
              ))}
            </select>
          </div>

          {ammo.loading ? (
            <p className="text-sm text-muted-foreground py-8 text-center">
              Loading ammo data...
            </p>
          ) : filteredAmmo.length > 0 ? (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead
                    className="cursor-pointer select-none"
                    onClick={() => toggleAmmoSort("name")}
                  >
                    Name{ammoSortIndicator("name")}
                  </TableHead>
                  <TableHead>Caliber</TableHead>
                  <TableHead
                    className="text-right cursor-pointer select-none"
                    onClick={() => toggleAmmoSort("damage")}
                  >
                    Damage{ammoSortIndicator("damage")}
                  </TableHead>
                  <TableHead
                    className="text-right cursor-pointer select-none"
                    onClick={() => toggleAmmoSort("penetration")}
                  >
                    Penetration{ammoSortIndicator("penetration")}
                  </TableHead>
                  <TableHead
                    className="text-right cursor-pointer select-none"
                    onClick={() => toggleAmmoSort("armor_damage")}
                  >
                    Armor Dmg{ammoSortIndicator("armor_damage")}
                  </TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {filteredAmmo.map((a) => (
                  <TableRow key={`${a.caliber}-${a.short_name}`}>
                    <TableCell>
                      <div>
                        <span className="font-medium">{a.short_name}</span>
                        <span className="text-xs text-muted-foreground ml-2">
                          {a.name}
                        </span>
                      </div>
                    </TableCell>
                    <TableCell>
                      <Badge variant="outline" className="text-xs">
                        {a.caliber}
                      </Badge>
                    </TableCell>
                    <TableCell className="text-right font-mono">
                      {a.damage}
                    </TableCell>
                    <TableCell
                      className={`text-right font-mono ${penColor(a.penetration)}`}
                    >
                      {a.penetration}
                    </TableCell>
                    <TableCell className="text-right font-mono">
                      {a.armor_damage}
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          ) : (
            <p className="text-sm text-muted-foreground py-8 text-center">
              No ammo data available
            </p>
          )}
        </CardContent>
      </Card>

      {/* Item Price Search */}
      <Card>
        <CardHeader>
          <CardTitle className="text-base">Flea Market Prices</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex gap-2">
            <input
              type="text"
              value={itemQuery}
              onChange={(e) => setItemQuery(e.target.value)}
              onKeyDown={(e) => e.key === "Enter" && searchItems()}
              placeholder="Search item by name..."
              className="flex-1 h-8 rounded-lg border border-border bg-background px-3 text-sm placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring"
            />
            <Button
              onClick={searchItems}
              disabled={itemSearching || !itemQuery.trim()}
            >
              {itemSearching ? "Searching..." : "Search"}
            </Button>
          </div>

          {itemResults.length > 0 && (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Item</TableHead>
                  <TableHead className="text-right">Avg 24h</TableHead>
                  <TableHead className="text-right">Last Low</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {itemResults.map((item) => (
                  <TableRow key={item.short_name}>
                    <TableCell>
                      <div>
                        <span className="font-medium">{item.short_name}</span>
                        <span className="text-xs text-muted-foreground ml-2">
                          {item.name}
                        </span>
                      </div>
                    </TableCell>
                    <TableCell className="text-right font-mono">
                      {formatPrice(item.avg_24h_price)}
                    </TableCell>
                    <TableCell className="text-right font-mono">
                      {formatPrice(item.last_low_price)}
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
